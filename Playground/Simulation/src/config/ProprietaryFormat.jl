module ProprietaryFormat
using Serde
using AutoHashEquals

struct Configuration
    topology::String
    scenario::String
end

function load(base_path, configuraion_file_name="main.json")
    if !endswith(base_path, "/")
        base_path *= "/"
    end

    config_file_path = base_path * configuraion_file_name
    config = open(config_file_path) do config_file
        deser_json(Configuration, read(config_file))
    end

    topology_file_path = base_path * config.topology
    scenario_file_path = base_path * config.scenario

    topology = open(topology_file_path) do topology_file
        deser_json(Topology, read(topology_file))
    end

    scenario = open(scenario_file_path) do scenario_file
        deser_json(Scenario, read(scenario_file))
    end

    (topology, scenario)
end

# Topology

@auto_hash_equals struct Node
    position::Vector{Float64}
    is_feed::Bool
end

@auto_hash_equals struct Pipe
    nodes::Vector{String}
    length::Float64
    diameter::Float64
    transmittance::Float64
    roughness::Float64
    zeta::Float64
end

@auto_hash_equals struct Consumer
    nodes::Vector{String}
end

@auto_hash_equals struct Source
    nodes::Vector{String}
end

@auto_hash_equals struct Topology
    nodes::Dict{String,Node}
    pipes::Dict{String,Pipe}
    consumers::Dict{String,Consumer}
    sources::Dict{String,Source}
end

# Scenario

@serde @de_name struct Settings
    feed_temperature::Float64 | "feed_temperature [C]"
    return_temperature::Float64 | "return_temperature [C]"
    ground_temperature::Float64 | "ground_temperature [C]"
    t_start::Float64 | "t_start [d]"
    t_end::Float64 | "t_end [d]"
    dt::Float64 | "dt [min]"
    ramp::Float64 | "ramp [h]"
    iter::Int64 | "num_iter"
    tolerance::Float64 | "tol"
end

struct Signal
    type::String
    axes::Vector{Vector{String}}
    unit_scale::Float64
    data::Union{Float64,Vector{Vector{Float64}}}
end

function Serde.deser(
    ::Type{<:Signal},
    ::Type{<:Union{Float64,Vector{Vector{Float64}}}},
    value,
)
    try
        Serde.deser(Vector{Vector{Float64}}, value)
    catch
        Serde.deser(Float64, value)
    end
end

struct Input
    signals::Vector{String}
end

struct ConsumerSignal
    return_temperature::Float64
    annual_consumption::Float64
    input::String
end

struct SourceSignal
    type::String
    input::String
end

struct PipeSignal
    input::String
end

struct Scenario
    settings::Settings
    signals::Dict{String,Signal}
    inputs::Dict{String,Input}
    consumers::Dict{String,ConsumerSignal}
    sources::Dict{String,SourceSignal}
    pipes::Dict{String,PipeSignal}
end

# bundle

struct DistrictHeatingNetwork
    topology::Topology
    scenario::Scenario
end

end # module ProprietaryFormat
