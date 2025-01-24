using Test

using Simulation

@testset verbose = true "parsing" begin
    include("Parsing/parsing_tests.jl")
end
