using Test

include("src/config/Configuration.jl")

@testset verbose = true "parsing" begin
    @testset "parse proprietary configuration" begin
        base_path = "./Data/Proprietary Format"
        @test try (topology, scenario) = Configuration.ProprietaryFormat.load(base_path)
            true
        catch
            false
        end
    end
end
