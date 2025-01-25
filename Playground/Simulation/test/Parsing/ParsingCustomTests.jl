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

@testset "correctly parsing custom node" begin
    test_json_deser(
        """{
            "name": "A",
            "position": {
                "x": 100,
                "y": 100,
                "z": 10
            },
            "feed": true
        }""",
        Simulation.Configuration.CustomFormat.Node(
            "A",
            Simulation.Configuration.CustomFormat.Position(
                100, 100, 10,
            ),
            true,
        ),
    )
end

@testset "correctly parsing custom pipe" begin
    test_json_deser(
        """{
            "name": "pipe1",
            "length": 100,
            "diameter": 2,
            "transmittance": 3,
            "roughness": 4,
            "zeta": 5,
            "src": "nodeA",
            "tgt": "nodeB"
        }""",
        Simulation.Configuration.CustomFormat.Pipe(
            "pipe1", 100, 2, 3, 4, 5, "nodeA", "nodeB",
        )
    )
end

@testset "correctly parsing custom consumer" begin
    test_json_deser(
        """{
            "name": "consumer1",
            "src": "nodeA",
            "tgt": "nodeB"
        }""",
        Simulation.Configuration.CustomFormat.Consumer(
            "consumer1", "nodeA", "nodeB",
        )
    )
end

@testset "correctly parsing custom source" begin
    test_json_deser(
        """{
            "name": "source1",
            "src": "nodeA",
            "tgt": "nodeB"
        }""",
        Simulation.Configuration.CustomFormat.Source(
            "source1", "nodeA", "nodeB",
        )
    )
end

@testset "correctly parsing custom settings" begin
    test_json_deser(
        """{
            "feed_temperature": 120,
            "return_temperature": 60,
            "ground_temperature": 1,
            "time_start": 0,
            "time_end": 2.4,
            "time_step": 15,
            "ramp_time": 8,
            "num_iterations": 1000,
            "tolerance": 1e-6
        }""",
        Simulation.Configuration.CustomFormat.Settings(
            120, 60, 1,
            0, 2.4, 15, 8,
            1000, 1e-6,
        )
    )
end

@testset "correctly parsing custom constant signal" begin
    test_json_deser(
        """{
            "name": "signal1",
            "scale": 1e6,
            "data": 1
        }""",
        Simulation.Configuration.CustomFormat.SignalConst(
            "signal1", 1e6, 1,
        )
    )
end

@testset "correctly parsing custom polynomial signal" begin
    test_json_deser(
        """{
            "name": "signal1",
            "degree": 1,
            "scale": 1e6,
            "data": [[0, 1], [1, 2]]
        }""",
        Simulation.Configuration.CustomFormat.SignalPoly(
            "signal1", 1, 1e6, [
                Simulation.Configuration.CustomFormat.DataPoint(0, 1),
                Simulation.Configuration.CustomFormat.DataPoint(1, 2),
            ],
        )
    )
end

@testset "correctly parsing custom input" begin
    test_json_deser(
        """{
            "name": "input1",
            "signals": ["signal1", "one"]
        }""",
        Simulation.Configuration.CustomFormat.Input(
            "input1", ["signal1", "one"],
        )
    )
end

@testset "correctly parsing custom consumer input mapping" begin
    test_json_deser(
        """{
            "name": "consumer1",
            "input": "input1",
            "return_temperature": 60,
            "annual_consumption": 2
        }""",
        Simulation.Configuration.CustomFormat.ConsumerInputMapping(
            "consumer1", "input1", 60, 2,
        )
    )
end

@testset "correctly parsing custom input mapping" begin
    test_json_deser(
        """{
            "name": "consumer1",
            "input": "input1"
        }""",
        Simulation.Configuration.CustomFormat.InputMapping(
            "consumer1", "input1",
        )
    )
end
