module Simulation

include("ProprietaryFormat.jl")
include("CustomFormat.jl")

function load()
    base_path = "./Data/Proprietary Format"
    @show ProprietaryFormat.load(base_path)

    base_path = "./Data/Custom Format"
    @show CustomFormat.load(base_path)
end


export load

end # module Simulation
