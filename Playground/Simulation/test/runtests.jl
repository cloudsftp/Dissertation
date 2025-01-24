using Test

using Simulation

@testset verbose = true "parsing" begin
    include("parsing_tests.jl")
end
