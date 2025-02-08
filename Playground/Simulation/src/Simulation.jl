module Simulation

using Serde

include("config/Configuration.jl")
include("Feed.jl")
include("SpanningTree.jl")

function load()
    #base_path = "./Data/Proprietary Format"
    #(topology, scenario) = Configuration.ProprietaryFormat.load(base_path)
    #@show ProprietaryFormat.load(base_path)

    base_path = "./Data/Custom Format"
    (; topology, scenario) = Configuration.CustomFormat.load(base_path)

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

    for i in 1:length(Q)
        consumer_name = topology.consumers[i].name
        demand_signal = scenario.consumers
        #Qᵢ = consumer.
        Q[i] = f * ()
    end

    Q
end

end # module Simulation
