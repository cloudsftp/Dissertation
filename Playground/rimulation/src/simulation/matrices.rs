use anyhow::{anyhow, Error};
use nalgebra::{stack, DMatrix, DVector};

use crate::{
    transition::transition_cubic,
    types::network::{Edge, Network},
    water,
};

fn reynold(edge: &Edge, e: f64, v: f64) -> f64 {
    v.abs() * edge.parameters.length / water::viscosity(e)
}

fn lambda(edge: &Edge, e: f64, v: f64) -> f64 {
    darcy_friction(edge, reynold(edge, e, v))
}

fn darcy_friction(edge: &Edge, re: f64) -> f64 {
    let laminar = 64. / re;

    // serghides's solution
    let turbulent = |re: f64| {
        let summand = edge.parameters.roughness / (3.7 * edge.parameters.diameter);

        let a = -(summand + 12. / re).log2();
        let b = -(summand + 2.51 * a / re).log2();
        let c = -(summand + 2.51 * b / re).log2();

        1. / (a - ((b - a).powi(2) / (c - 2. * b + a))).powi(2)
    };

    let left_boundary = 2_000.;
    let right_boundary = 4_000.;

    if re > right_boundary {
        turbulent(re)
    } else if re < left_boundary {
        laminar
    } else {
        let right_boundary_value = turbulent(right_boundary);
        let dre = 1e-3;
        let right_boundary_slope = (turbulent(right_boundary + dre) - right_boundary_value) / dre;

        transition_cubic(
            re,
            left_boundary,
            right_boundary,
            laminar,
            0.,
            right_boundary_value,
            right_boundary_slope,
        )
    }
}

fn lambda_mat(network: &Network, e: DVector<f64>, v: DVector<f64>) -> DMatrix<f64> {
    let lambda = DVector::from_iterator(
        network.num_edges(),
        network.edges().enumerate().map(|(i, edge)| {
            let v = v[i];
            let e = (e[edge.src] + e[edge.tgt]) / 2.;

            lambda(edge, e, v)
        }),
    );

    DMatrix::from_diagonal(&lambda)
}

fn ar(network: &Network) -> DMatrix<f64> {
    DMatrix::from_iterator(
        network.num_edges(),
        network.demand_nodes.len(),
        (0..network.demand_nodes.len())
            .map(|i| {
                network.edges().map(move |edge| {
                    if edge.src == i {
                        -1.
                    } else if edge.tgt == i {
                        1.
                    } else {
                        0.
                    }
                })
            })
            .flatten(),
    )
}

fn arp(network: &Network) -> DMatrix<f64> {
    let num_pressure_nodes = network.pressure_nodes.len();
    let num_demand_nodes = network.demand_nodes.len();

    DMatrix::from_iterator(
        network.num_edges(),
        num_pressure_nodes,
        (0..num_pressure_nodes)
            .map(|i| {
                let i = num_demand_nodes + i;
                network.edges().map(move |edge| {
                    if edge.src == i {
                        -1.
                    } else if edge.tgt == i {
                        1.
                    } else {
                        0.
                    }
                })
            })
            .flatten(),
    )
}

fn ai(network: &Network) -> DMatrix<f64> {
    let ar = ar(network);
    let arp = arp(network);

    stack![ar, arp]
}

fn at(network: &Network) -> DMatrix<f64> {
    DMatrix::from_iterator(
        network.demand_nodes.len(),
        network.num_edges(),
        (0..network.num_edges())
            .map(|i| (0..network.demand_nodes.len()).map(move |j| if i == j { 1. } else { 0. }))
            .flatten(),
    )
}

fn ac(network: &Network) -> Result<DMatrix<f64>, Error> {
    let mut ac = DMatrix::from_element(network.num_cycles(), network.num_nodes(), 0.);
    let mut set_matrix_element = |i, j, v| {
        let index = j * network.num_cycles() + i;
        if index < network.num_cycles() * network.num_nodes() {
            ac[index] = v;
            Ok(())
        } else {
            Err(anyhow!(
                "index out of range when setting matrix element {}, {}",
                i,
                j,
            ))
        }
    };

    for (i, cycle_edge) in network.cycle_edges.iter().enumerate() {
        let j = network.spanning_tree_edges.len() + i;
        set_matrix_element(i, j, 1.)?;

        let mut walk_cycle = |c: &usize, invert| -> Result<(), Error> {
            let mut c = c;
            while *c != network.root_node_index {
                let p = network
                    .pred_nodes
                    .get(c)
                    .ok_or(anyhow!("could not find predecessor of node {}", c))?;

                let (j, reversed) = network
                    .edge_indices_by_connected_nodes
                    .get(&(*p, *c))
                    .ok_or(anyhow!(
                        "could not get the index of the edge connecting the nodes {} and {}",
                        c,
                        p,
                    ))?;

                set_matrix_element(i, *j, if *reversed != invert { -1. } else { 1. })?;

                c = p;
            }

            Ok(())
        };

        walk_cycle(&cycle_edge.src, false)?;
        walk_cycle(&cycle_edge.tgt, true)?;
    }

    Ok(ac)
}

fn dinv(network: &Network) -> DMatrix<f64> {
    DMatrix::from_diagonal(&DVector::from_iterator(
        network.num_edges(),
        network.edges().map(|edge| 1. / edge.parameters.diameter),
    ))
}

pub struct Matrices {
    pub ai: DMatrix<f64>,
    pub ar: DMatrix<f64>,
    pub at: DMatrix<f64>,
    pub ac: DMatrix<f64>,
}

impl TryFrom<&Network> for Matrices {
    type Error = Error;

    fn try_from(network: &Network) -> Result<Self, Self::Error> {
        let ar = ar(network);
        let arp = arp(network);
        let ai = ai(network);
        let at = at(network);
        let ac = ac(network)?;

        Ok(Self { ai, ar, at, ac })
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, io::Write};

    use super::*;

    use crate::types::network::{
        test::{DUMMY_CONST_SIGNAL, DUMMY_PIPE_PARAMETERS},
        Edge, Node, PipeParameters,
    };

    fn create_test_net() -> Network {
        let nodes = (0..4)
            .map(|i| Node::Zero {
                name: format!("N{}", i),
            })
            .chain(
                [Node::Pressure {
                    name: String::from("N4"),
                    pressure: DUMMY_CONST_SIGNAL,
                    temperature: DUMMY_CONST_SIGNAL,
                }]
                .into_iter(),
            )
            .collect();
        let edges = [(0, 4), (0, 1), (1, 2), (3, 2), (3, 4)]
            .map(|(i, j)| Edge {
                src: i,
                tgt: j,
                parameters: DUMMY_PIPE_PARAMETERS,
            })
            .to_vec();

        Network::try_from_feed(nodes, edges)
            .expect("could not compute network from feed nodes and edges")
    }

    #[test]
    fn compute_ac() {
        let nodes = (0..5)
            .map(|i| Node::Zero {
                name: format!("N{}", i),
            })
            .collect();
        let edges = [(1, 0), (1, 2), (2, 3), (4, 3), (4, 0)]
            .map(|(i, j)| Edge {
                src: i,
                tgt: j,
                parameters: DUMMY_PIPE_PARAMETERS,
            })
            .to_vec();

        let network = Network::try_from_feed(nodes, edges)
            .expect("could not compute network from feed nodes and edges");

        let ac = ac(&network).expect("could not compute A_C matrix");
        assert_eq!(ac, DMatrix::from_vec(1, 5, vec![-1., 1., -1., 1., 1.]));
    }

    #[test]
    fn compute_ar() {
        let network = create_test_net();

        let ar = ar(&network);
        assert_eq!(
            ar,
            DMatrix::from_row_slice(
                5,
                4,
                &[
                    [-1., 0., 0., 0.], // edge 0
                    [-1., 1., 0., 0.], // edge 1
                    [0., 0., 1., -1.], // edge 3
                    [0., 0., 0., -1.], // edge 4
                    [0., -1., 1., 0.], // edge 2
                ]
                .concat()
            )
        )
    }

    #[test]
    fn compute_arp() {
        let network = create_test_net();

        let arp = arp(&network);
        assert_eq!(arp, DMatrix::from_row_slice(5, 1, &[1., 0., 0., 1., 0.]))
    }

    #[test]
    fn compute_ai() {
        let network = create_test_net();

        let ai = ai(&network);
        assert_eq!(
            ai,
            DMatrix::from_row_slice(
                5,
                5,
                &[
                    [-1., 0., 0., 0., 1.], // edge 0
                    [-1., 1., 0., 0., 0.], // edge 1
                    [0., 0., 1., -1., 0.], // edge 3
                    [0., 0., 0., -1., 1.], // edge 4
                    [0., -1., 1., 0., 0.], // edge 2
                ]
                .concat()
            )
        )
    }

    #[test]
    fn compute_at() {
        let nodes = (0..4)
            .map(|i| Node::Zero {
                name: format!("N{}", i),
            })
            .collect();
        let edges = [(1, 0), (1, 2), (2, 3), (1, 3), (3, 0), (1, 2)]
            .map(|(i, j)| Edge {
                src: i,
                tgt: j,
                parameters: DUMMY_PIPE_PARAMETERS,
            })
            .to_vec();

        let network = Network::try_from_feed(nodes, edges)
            .expect("could not compute network from feed nodes and edges");

        let at = at(&network);
        assert_eq!(
            at,
            DMatrix::from_row_slice(
                4,
                6,
                &[
                    [1., 0., 0., 0., 0., 0.],
                    [0., 1., 0., 0., 0., 0.],
                    [0., 0., 1., 0., 0., 0.],
                    [0., 0., 0., 1., 0., 0.],
                ]
                .concat()
            )
        )
    }

    #[test]
    fn generate_darcy_friction_factors() {
        let mut file = fs::File::create("/tmp/darcey_friction_data").expect("could not open file");

        let n = 10_000usize;

        let l = 1_000.;
        let r = 10_000.;

        let edge = &Edge {
            src: 0,
            tgt: 1,
            parameters: PipeParameters {
                length: 1.,
                diameter: 0.2,
                transmittance: 1.,
                roughness: 1e-3,
                zeta: 1.,
            },
        };

        for i in 0..n {
            let re = l + i as f64 * (r - l) / n as f64;
            let lambda = darcy_friction(edge, re);

            file.write(format!("{} {}\n", re, lambda).as_bytes())
                .expect("could not write to file");
        }
    }
}
