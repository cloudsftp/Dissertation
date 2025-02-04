module Simulation

using Serde

include("config/Configuration.jl")
include("Feed.jl")

function load()
    #base_path = "./Data/Proprietary Format"
    #(topology, scenario) = Configuration.ProprietaryFormat.load(base_path)
    #@show ProprietaryFormat.load(base_path)

    base_path = "./Data/Custom Format"
    (; topology, scenario) = Configuration.CustomFormat.load(base_path)

    @show compute_feed(topology)
end

end # module Simulation
