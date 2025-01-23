module Simulation

using Serde

include("config/Configuration.jl")


function load()
    base_path = "./Data/Proprietary Format"
    (topology, scenario) = Configuration.ProprietaryFormat.load(base_path)
    #@show ProprietaryFormat.load(base_path)

    base_path = "./Data/Custom Format"
    #(; topology, scenario) = CustomFormat.load(base_path)

    #@show to_json(scenario)

    @show Configuration.to_custom(scenario.signals)
end


export load

end # module Simulation
