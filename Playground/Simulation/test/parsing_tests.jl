using Serde

function test_json_deser(json, expected)
    @test deser_json(typeof(expected), json) == expected
end

@testset "no exception when parsing proprietary configuration example" begin

    base_path = "../Data/Proprietary Format"
    @test try
        (topology, scenario) = Simulation.Configuration.ProprietaryFormat.load(base_path)
        true
    catch e
        @show e
        false
    end
end

@testset "correctly parsing proprietary configuration file" begin
    test_json_deser(
        """{
            "topology": "a",
            "scenario": "b"
        }""",
        Simulation.Configuration.ProprietaryFormat.Configuration("a", "b"),
    )
end

@testset "correctly parsing proprietary node" begin
    test_json_deser(
        """{
            "position": [100.0, 2, 3],
            "is_feed": true
        }""",
        Simulation.Configuration.ProprietaryFormat.Node([100, 2, 3], true),
    )
end

@testset "correctly parsing proprietary pipe" begin
    test_json_deser(
        """{
            "nodes": ["A", "B"],
            "length": 1.0,
            "diameter": 2e0,
            "transmittance": 3,
            "roughness": 4,
            "zeta": 5
        }""",
        Simulation.Configuration.ProprietaryFormat.Pipe(["A", "B"], 1, 2, 3, 4, 5),
    )
end

@testset "correctly parsing proprietary consumer" begin
    test_json_deser(
        """{
            "nodes": ["A", "B"]
        }""",
        Simulation.Configuration.ProprietaryFormat.Consumer(["A", "B"])
    )
end

@testset "correctly parsing proprietary source" begin
    test_json_deser(
        """{
            "nodes": ["A", "B"]
        }""",
        Simulation.Configuration.ProprietaryFormat.Source(["A", "B"])
    )
end

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
