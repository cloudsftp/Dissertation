module Simulation

using Serde

include("config/Configuration.jl")
include("Feed.jl")
include("SpanningTree.jl")
include("Signals.jl")

function load()
    base_path = "./Data/Custom Format"
    (; topology, scenario) = Configuration.CustomFormat.load(base_path)

    signals = Signals(scenario)

    feed = compute_feed(topology)
    (tree, cycles, pred) = find_spanning_tree(feed)

    Q = compute_modified_demand(topology, scenario)
    @show Q

    v₀ = [0.]
end

# for now only first time step
function compute_modified_demand(topology, scenario)
    Q = Vector{Float64}(undef, length(topology.consumers))

    α = 1.
    f = 1. / α
    @show scenario.consumer_inputs

    for i in 1:length(Q)
        consumer_name = topology.consumers[i].name
        consumer_input = scenario.consumer_inputs[consumer_name]

        demand_signal_name = scenario.inputs[consumer_input.input].power
        demand_signal = scenario.signals[demand_signal_name]

        @show demand_signal

        #Qᵢ = consumer.
        #Q[i] = f * ()
    end

    Q
end

end # module Simulation
