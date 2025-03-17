use anyhow::Error;
use nalgebra::{stack, DMatrix, DVector};

use crate::{
    transition::transition_cubic,
    types::network::{HydraulicPipeParameters, Network},
    water,
};

fn reynold(edge: &impl HydraulicPipeParameters, e: f64, v: f64) -> f64 {
    v.abs() * edge.length() / water::viscosity(e)
}

// Reynolds number transition boundaries for laminar to turbulent flow
const DARCY_LEFT_BOUNDARY: f64 = 2_000.;
const DARCY_RIGHT_BOUNDARY: f64 = 4_000.;

/// Calculates the Darcy friction factor using:
/// - Laminar flow (Re < 2000): f = 64/Re
/// - Turbulent flow (Re > 4000): Serghide's solution
/// - Transition (2000 <= Re <= 4000): Cubic interpolation
fn darcy_friction(edge: &impl HydraulicPipeParameters, re: f64) -> f64 {
    let laminar = |re| 64. / re;

    // serghides's solution
    let turbulent = |re: f64| {
        let summand = edge.roughness() / (3.7 * edge.diameter());

        let a = -(summand + 12. / re).log2();
        let b = -(summand + 2.51 * a / re).log2();
        let c = -(summand + 2.51 * b / re).log2();

        1. / (a - ((b - a).powi(2) / (c - 2. * b + a))).powi(2)
    };

    if re > DARCY_RIGHT_BOUNDARY {
        turbulent(re)
    } else if re < DARCY_LEFT_BOUNDARY {
        laminar(re)
    } else {
        let dre = 1e-3;

        let left_boundary_value = laminar(DARCY_LEFT_BOUNDARY);
        let left_boundary_slope = (laminar(DARCY_LEFT_BOUNDARY + dre) - left_boundary_value) / dre;

        let right_boundary_value = turbulent(DARCY_RIGHT_BOUNDARY);
        let right_boundary_slope =
            (turbulent(DARCY_RIGHT_BOUNDARY + dre) - right_boundary_value) / dre;

        transition_cubic(
            re,
            DARCY_LEFT_BOUNDARY,
            DARCY_RIGHT_BOUNDARY,
            left_boundary_value,
            left_boundary_slope,
            right_boundary_value,
            right_boundary_slope,
        )
    }
}

fn lambda<PipeParameters>(
    network: &Network<PipeParameters>,
    e: DVector<f64>,
    v: DVector<f64>,
) -> DMatrix<f64>
where
    PipeParameters: HydraulicPipeParameters,
{
    let lambda = DVector::from_iterator(
        network.num_edges(),
        network
            .edges()
            .zip(network.edge_parameters())
            .enumerate()
            .map(|(i, (edge, edge_parameters))| {
                let v = v[i];
                let e = (e[edge.src] + e[edge.tgt]) / 2.;

                darcy_friction(edge_parameters, reynold(edge_parameters, e, v))
            }),
    );

    DMatrix::from_diagonal(&lambda)
}

fn ar<PipeParameters>(network: &Network<PipeParameters>) -> DMatrix<f64> {
    DMatrix::from_iterator(
        network.num_edges(),
        network.demand_nodes.len(),
        (0..network.demand_nodes.len())
            .flat_map(|i| {
                network.edges().map(move |edge| {
                    if edge.src == i {
                        -1.
                    } else if edge.tgt == i {
                        1.
                    } else {
                        0.
                    }
                })
            }),
    )
}

fn arp<PipeParameters>(network: &Network<PipeParameters>) -> DMatrix<f64> {
    let num_pressure_nodes = network.pressure_nodes.len();
    let num_demand_nodes = network.demand_nodes.len();

    DMatrix::from_iterator(
        network.num_edges(),
        num_pressure_nodes,
        (0..num_pressure_nodes)
            .flat_map(|i| {
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
            }),
    )
}

fn ai<PipeParameters>(network: &Network<PipeParameters>) -> DMatrix<f64> {
    let ar = ar(network);
    let arp = arp(network);

    stack![ar, arp]
}

fn at<PipeParameters>(network: &Network<PipeParameters>) -> DMatrix<f64> {
    DMatrix::from_iterator(
        network.demand_nodes.len(),
        network.num_edges(),
        (0..network.num_edges())
            .flat_map(|i| (0..network.demand_nodes.len()).map(move |j| if i == j { 1. } else { 0. })),
    )
}

#[derive(Debug, thiserror::Error)]
enum MatrixError {
    #[error("Index out of bounds when setting matrix element: ({i}, {j})")]
    IndexOutOfBounds { i: usize, j: usize },
    #[error("Missing predecessor for node {0}")]
    MissingPredecessor(usize),
    #[error("Missing edge between nodes {0} and {1}")]
    MissingEdge(usize, usize),
}

fn ac<PipeParameters>(network: &Network<PipeParameters>) -> Result<DMatrix<f64>, Error> {
    let mut ac = DMatrix::from_element(network.num_cycles(), network.num_nodes(), 0.);
    let mut set_matrix_element = |i, j, v| {
        (i < ac.nrows() && j < ac.ncols())
            .then(|| ac[(i, j)] = v)
            .ok_or(MatrixError::IndexOutOfBounds { i, j })
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
                    .ok_or(MatrixError::MissingPredecessor(*c))?;

                let (j, reversed) = network
                    .edge_indices_by_connected_nodes
                    .get(&(*p, *c))
                    .ok_or(MatrixError::MissingEdge(*p, *c))?;

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

fn dinv<PipeParameters>(network: &Network<PipeParameters>) -> DMatrix<f64>
where
    PipeParameters: HydraulicPipeParameters,
{
    DMatrix::from_diagonal(&DVector::from_iterator(
        network.num_edges(),
        network
            .edge_parameters()
            .map(|edge_parameters| 1. / edge_parameters.diameter()),
    ))
}

/// Stores the matrices used in hydraulic and thermal network calculations
pub struct Matrices {
    /// Incidence matrix for the entire network
    pub ai: DMatrix<f64>,
    /// Reduced incidence matrix for demand nodes
    pub ar: DMatrix<f64>,
    /// Reduced incidence matrix for pressure nodes
    pub arp: DMatrix<f64>,
    /// Identity matrix of all demand nodes
    /// concatenated with 0s to have as much columns as the network has edges
    pub at: DMatrix<f64>,
    /// Cycle incidence matrix (includes information about orientation)
    pub ac: DMatrix<f64>,
}

impl<T> TryFrom<&Network<T>> for Matrices {
    type Error = Error;

    fn try_from(network: &Network<T>) -> Result<Self, Self::Error> {
        let ar = ar(network);
        let arp = arp(network);
        let ai = ai(network);
        let at = at(network);
        let ac = ac(network)?;

        Ok(Self {
            ai,
            ar,
            arp,
            at,
            ac,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, io::Write};

    use super::*;

    use crate::types::network::{EmptyPipeParameters, FullPipeParameters};
    use crate::types::{
        formats::custom::test_util::DUMMY_CUSTOM_POSITION,
        network::{
            test::{DUMMY_CONST_SIGNAL, DUMMY_PIPE_PARAMETERS},
            Edge, Node,
        },
    };

    fn create_test_net() -> Network<EmptyPipeParameters> {
        let nodes = (0..4)
            .map(|i| Node::Zero {
                name: format!("N{}", i),
                position: DUMMY_CUSTOM_POSITION,
            })
            .chain(
                [Node::Pressure {
                    name: String::from("N4"),
                    pressure: DUMMY_CONST_SIGNAL,
                    temperature: DUMMY_CONST_SIGNAL,
                    position: DUMMY_CUSTOM_POSITION,
                }],
            )
            .collect();
        let edges = [(0, 4), (0, 1), (1, 2), (3, 2), (3, 4)]
            .map(|(src, tgt)| Edge {
                src,
                tgt,
                // parameters: DUMMY_PIPE_PARAMETERS,
            })
            .to_vec();

        let edge_parameters = (0..edges.len()).map(|_| DUMMY_PIPE_PARAMETERS).collect();

        Network::try_from_feed(nodes, edges, edge_parameters)
            .expect("could not compute network from feed nodes and edges")
    }

    #[test]
    fn compute_ac() {
        let nodes = (0..5)
            .map(|i| Node::Zero {
                name: format!("N{}", i),
                position: DUMMY_CUSTOM_POSITION,
            })
            .collect();
        let edges = [(1, 0), (1, 2), (2, 3), (4, 3), (4, 0)]
            .map(|(src, tgt)| Edge { src, tgt })
            .to_vec();

        let edge_parameters = (0..edges.len()).map(|_| DUMMY_PIPE_PARAMETERS).collect();

        let network = Network::try_from_feed(nodes, edges, edge_parameters)
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
                position: DUMMY_CUSTOM_POSITION,
            })
            .collect();
        let edges = [(1, 0), (1, 2), (2, 3), (1, 3), (3, 0), (1, 2)]
            .map(|(src, tgt)| Edge { src, tgt })
            .to_vec();

        let edge_parameters = (0..edges.len()).map(|_| DUMMY_PIPE_PARAMETERS).collect();

        let network = Network::try_from_feed(nodes, edges, edge_parameters)
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

        let edge_parameters = &FullPipeParameters {
            length: 1.,
            diameter: 0.2,
            transmittance: 1.,
            roughness: 1e-2,
            zeta: 1.,
        };

        for i in 0..n {
            let re = l + i as f64 * (r - l) / n as f64;
            let lambda = darcy_friction(edge_parameters, re);

            file.write(format!("{} {}\n", re, lambda).as_bytes())
                .expect("could not write to file");
        }
    }
}
