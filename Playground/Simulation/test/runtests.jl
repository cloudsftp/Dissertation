using Test

using Simulation

@testset verbose = true "parsing" begin
    @testset "parse proprietary configuration" begin
        base_path = "../Data/Proprietary Format"
        @test try (topology, scenario) = Simulation.Configuration.ProprietaryFormat.load(base_path)
            true
        catch e
            @show e
            false
        end
    end
end
