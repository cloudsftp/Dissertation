using Test

using Simulation

@testset verbose = true "parsing" begin
    include("Parsing/parsing_tests.jl")
end

@testset verbose = true "converting" begin
    include("FormatConversion/conversion_tests.jl")
end
