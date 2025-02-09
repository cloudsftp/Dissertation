module CustomFormat

using Serde
using AutoHashEquals

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

function get_other_node(pipe::Pipe, node::String)
    if pipe.src == node
        pipe.tgt
    elseif pipe.tgt == node
        pipe.src
    else
        throw(ErrorException("pipe " * pipe * " does not connect to node " * node))
    end
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

@auto_hash_equals struct Topology
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

struct SignalConst
    name::String
    scale::Float64
    data::Float64
end

struct DataPoint
    t::Float64
    v::Float64
end

@auto_hash_equals struct SignalPoly
    name::String
    degree::UInt8
    scale::Float64
    data::Vector{DataPoint}
end

struct ConsumerSignals
    name::String
    power::String
    return_temperature::String
end

struct SourceSignals
    name::String
    base_pressure::String
    pressure_lift::String
    temperature::String
end

struct UnknownSignals
    name::String
    signals::Vector{String}
end

struct ConsumerSignalFactors
    yearly_power_demand::Float64
    normal_return_temperature::Float64
end

struct ConsumerInput
    input::String
    factors::ConsumerSignalFactors
end

#struct PipeSignals
#    name::String
#    length_signal::String
#    diameter_signal::String
#    transmittance_signal::String
#    roughness_signal::String
#    zeta_signal::String
#end

struct Scenario
    settings::Settings
    signals::Dict{String,Union{SignalConst,SignalPoly}}
    inputs::Dict{String,Union{ConsumerSignals,SourceSignals,UnknownSignals}}
    consumer_inputs::Dict{String,ConsumerInput}
    source_inputs::Dict{String,String}
end

function Serde.deser(
    ::Type{<:Scenario},
    ::Type{<:Dict{String,Union{SignalConst,SignalPoly}}},
    list,
)
    list_to_dict(
        signal -> signal["name"],
        signal -> try
            Serde.deser(SignalPoly, signal)
        catch
            Serde.deser(SignalConst, signal)
        end,
        list,
    )
end

function Serde.deser(
    ::Type{<:Scenario},
    ::Type{<:Dict{String,Union{ConsumerSignals,SourceSignals,UnknownSignals}}},
    list,
)
    list_to_dict(
        input -> input["name"],
        input -> try
            Serde.deser(ConsumerSignals, input)
        catch
            Serde.deser(SourceSignals, input)
        end,
        list,
    )
end

function list_to_dict(key, value, list)
    result = Dict()

    for element in list
        result[key(element)] = value(element)
    end

    result
end

#

struct DistrictHeatingNetwork
    topology::Topology
    scenario::Scenario
end

end # module CustomFormat
