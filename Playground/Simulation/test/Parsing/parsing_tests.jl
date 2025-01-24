using Serde

@testset "proprietary" begin
    include("parsing_proprietary_tests.jl")
end

@testset "custom" begin
    include("parsing_custom_tests.jl")
end
