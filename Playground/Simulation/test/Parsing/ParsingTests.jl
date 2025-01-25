using Serde

@testset "proprietary" begin
    include("ParsingProprietaryTests.jl")
end

@testset "custom" begin
    include("ParsingCustomTests.jl")
end
