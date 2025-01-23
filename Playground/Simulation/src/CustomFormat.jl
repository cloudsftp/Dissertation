module CustomFormat

using Serde

function load(base_path)
    if !endswith(base_path, "/")
        base_path *= "/"
    end

    topology_file_name = base_path * "topology.json"
    scenario_file_name = base_path * "scenario.json"

    topology = open(topology_file_name) do topology_file
        deser_json(Topology, read(topology_file))
    end

    scenario = open(scenario_file_name) do scenario_file
        deser_json(Scenario, read(scenario_file))
    end

    DistrictHeatingNetwork(topology, scenario)
end

# Topology

struct Position
    x::Float64
    y::Float64
    z::Float64
end

struct Node
    name::String
    position::Position
    feed::Bool
end

struct Pipe
    name::String
    length::Float64
    diameter::Float64
    transmittance::Float64
    roughness::Float64
    zeta::Float64
    src::String
    tgt::String
end

struct Consumer
    name::String
    src::String
    tgt::String
end

struct Source
    name::String
    src::String
    tgt::String
end

struct Topology
    nodes::Vector{Node}
    pipes::Vector{Pipe}
    consumers::Vector{Consumer}
    sources::Vector{Source}
end

# Scenario

struct Settings # TODO: investigate which settings I don't need
    feed_temperature::Float64
    return_temperature::Float64
    ground_temperature::Float64
    time_start::Float64
    time_end::Float64
    time_step::Float64
    ramp_time::Float64
    num_iterations::UInt64
    tolerance::Float64
end

struct DataPoint
    t::Float64
    v::Float64
end

struct Signal
    name::String
    degree::Union{UInt8,Nothing}
    scale::Float64
    data::Union{Float64,Vector{DataPoint}}
end

struct Input
    name::String
    signals::Vector{String}
end

struct ConsumerInputMapping
    name::String
    input::String
    return_temperature::Float64
    annual_consumption::Float64
end

struct InputMapping
    name::String
    input::String
end

struct Scenario
    settings::Settings
    signals::Vector{Signal}
    inputs::Vector{Input}
    consumers::Vector{ConsumerInputMapping}
    sources::Vector{InputMapping}
    pipes::Vector{InputMapping}
end

function Serde.deser(
    ::Type{<:Signal},
    ::Type{<:Union{Float64,Vector{DataPoint}}},
    x,
)
    try
        Serde.deser(Vector{DataPoint}, x)
    catch
        Serde.deser(Float64, x)
    end
end

#

struct DistrictHeatingNetwork
    topology::Topology
    scenario::Scenario
end

end # module CustomFormat
