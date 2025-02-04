using Test

using Simulation

@testset verbose = true "parsing" begin
    include("Parsing/ParsingTests.jl")
end

@testset verbose = true "converting" begin
    include("FormatConversion/ConversionTests.jl")
end

@testset verbose = false "computing feed" begin
    include("ComputeFeed/ComputeFeedTests.jl")
end
