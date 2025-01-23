module Simulation

using Serde

include("ProprietaryFormat.jl")
include("CustomFormat.jl")

function load()
    #base_path = "./Data/Proprietary Format"
    #@show ProprietaryFormat.load(base_path)

    base_path = "./Data/Custom Format"
    (; topology, scenario) = CustomFormat.load(base_path)

    @show to_json(scenario)
end


export load

end # module Simulation
