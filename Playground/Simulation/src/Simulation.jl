module Simulation
using Serde

function load()
    base_path = "./Data/Proprietary Format"

    topology_file_path = base_path * "/topology.json"
    scenario_file_path = base_path * "/scenario.json"

    topology = open(topology_file_path) do topology_file
        deser_json(Topology, read(topology_file))
    end

    @show topology
end

@serde struct Node
    position::Vector{Float64}
    is_feed::Bool
end

@serde struct Pipe
    nodes::Vector{String}
    length::Float64
    diameter::Float64
    transmittance::Float64
    roughness::Float64
    zeta::Float64
end

@serde struct Consumer
    nodes::Vector{String}
end

@serde struct Producer
    nodes::Vector{String}
end

struct Topology
    nodes::Dict{String, Node}
    pipes::Dict{String, Pipe}
    consumers::Dict{String, Consumer}
    sources::Dict{String, Producer}
end

export load

end # module Simulation
