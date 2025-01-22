module Simulation

include("ProprietaryFormat.jl")

function load()
    base_path = "./Data/Proprietary Format"

    topology_file_path = base_path * "/topology.json"
    scenario_file_path = base_path * "/scenario.json"

    @show ProprietaryFormat.load(topology_file_path, scenario_file_path)
end


export load

end # module Simulation
