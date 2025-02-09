using Test

using Simulation

const PF = Simulation.Configuration.ProprietaryFormat
const CF = Simulation.Configuration.CustomFormat

run_all_tests = true
if length(ARGS) == 1 && ARGS[1] == "dev"
    run_all_tests = false
end

@testset verbose = true begin

    if run_all_tests

        @testset verbose = true "converting" begin
            include("FormatConversion/ConversionTests.jl")
        end

        @testset verbose = true "parsing" begin
            include("Parsing/ParsingTests.jl")
        end

        @testset "computing spanning tree" begin
            include("SpanningTreeTests.jl")
        end

        @testset "computing feed" begin
            include("ComputeFeedTests.jl")
        end

    end

end
