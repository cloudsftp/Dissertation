use super::*;

pub const DUMMY_CUSTOM_POSITION: Position = Position {
    x: 1.,
    y: 2.,
    z: 3.,
};

pub const DUMMY_CUSTOM_SETTINGS: Settings = Settings {
    feed_temperature: 1.,
    return_temperature: 2.,
    ground_temperature: 3.,
    time_start: 4.,
    time_end: 5.,
    time_step: 6.,
    ramp_time: 7.,
    num_iterations: 8,
    tolerance: 9.,
};

pub const DUMMY_CONST_CUSTOM_SIGNAL: Signal = Signal::Const {
    scale: 1.,
    data: 1.,
};

pub const DUMMY_CONSUMER_FACTORS: ConsumerSignalFactors = ConsumerSignalFactors {
    yearly_demand: 1.,
    normal_return_temperature: 1.,
};

fn create_test_topology(
    num_nodes: usize,
    num_feed_nodes: usize,
    edges: &[(usize, usize)],
    consumers: &[usize],
    sources: &[usize],
) -> Topology {
    let nodes = (0..num_nodes)
        .map(|i| Node {
            name: format!("N{}", i),
            position: DUMMY_CUSTOM_POSITION,
            feed: i < num_feed_nodes,
        })
        .collect();

    let pipes = edges
        .iter()
        .enumerate()
        .map(|(i, (src, tgt))| Pipe {
            name: format!("P{}", i),
            length: 1.,
            diameter: 2.,
            transmittance: 3.,
            roughness: 4.,
            zeta: 5.,
            src: format!("N{}", src),
            tgt: format!("N{}", tgt),
        })
        .collect();

    let consumers = consumers
        .iter()
        .enumerate()
        .map(|(i, j)| Consumer {
            name: format!("C{}", i),
            src: format!("N{}", j),
            tgt: format!("N{}", j + num_feed_nodes),
        })
        .collect();

    let sources = sources
        .iter()
        .enumerate()
        .map(|(i, j)| Source {
            name: format!("S{}", i),
            src: format!("N{}", j + num_feed_nodes),
            tgt: format!("N{}", j),
        })
        .collect();

    Topology {
        nodes,
        pipes,
        consumers,
        sources,
    }
}

fn create_test_scenario(num_consumers: usize, num_sources: usize) -> Scenario {
    let consumer_inputs = (0..num_consumers)
        .map(|i| {
            (
                format!("C{}", i),
                ConsumerInput {
                    input: String::from("fake_consumer_input"),
                    factors: DUMMY_CONSUMER_FACTORS,
                },
            )
        })
        .collect();

    let source_inputs = (0..num_sources)
        .map(|i| (format!("S{}", i), String::from("fake_source_input")))
        .collect();

    Scenario {
        settings: DUMMY_CUSTOM_SETTINGS,
        signals: [(String::from("const"), DUMMY_CONST_CUSTOM_SIGNAL)]
            .iter()
            .cloned()
            .collect(),
        inputs: [
            (
                String::from("fake_consumer_input"),
                Input::Consumer {
                    demand: String::from("const"),
                    return_temperature: String::from("const"),
                },
            ),
            (
                String::from("fake_source_input"),
                Input::Source {
                    base_pressure: String::from("const"),
                    pressure_lift: String::from("const"),
                    temperature: String::from("const"),
                },
            ),
        ]
        .iter()
        .cloned()
        .collect(),
        consumer_inputs,
        source_inputs,
    }
}

pub fn create_test_net(
    num_nodes: usize,
    num_feed_nodes: usize,
    edges: &[(usize, usize)],
    consumers: &[usize],
    sources: &[usize],
) -> Network {
    let topology = create_test_topology(num_nodes, num_feed_nodes, edges, consumers, sources);
    let scenario = create_test_scenario(consumers.len(), sources.len());

    Network { topology, scenario }
}
