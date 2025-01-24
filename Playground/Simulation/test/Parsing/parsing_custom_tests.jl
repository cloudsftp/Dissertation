using Serde

@testset "no exception when parsing custom configuration example" begin
    base_path = "../Data/Custom Format"
    @test try
        dhn = Simulation.Configuration.CustomFormat.load(base_path)
        true
    catch e
        @show e
        false
    end
end
