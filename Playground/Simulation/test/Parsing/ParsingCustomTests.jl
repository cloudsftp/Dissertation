@testset "no exception when parsing custom configuration example" begin
    base_path = "../Data/Custom Format"
    @test try
        CF.load(base_path)
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
        CF.Node("A", CF.Position(100, 100, 10), true),
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
        CF.Pipe("pipe1", 100, 2, 3, 4, 5, "nodeA", "nodeB")
    )
end

@testset "correctly parsing custom consumer" begin
    test_json_deser(
        """{
            "name": "consumer1",
            "src": "nodeA",
            "tgt": "nodeB"
        }""",
        CF.Consumer("consumer1", "nodeA", "nodeB")
    )
end

@testset "correctly parsing custom source" begin
    test_json_deser(
        """{
            "name": "source1",
            "src": "nodeA",
            "tgt": "nodeB"
        }""",
        CF.Source("source1", "nodeA", "nodeB")
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
        CF.Settings(
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
        CF.SignalConst("signal1", 1e6, 1)
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
        CF.SignalPoly(
            "signal1", 1, 1e6, [
                CF.DataPoint(0, 1),
                CF.DataPoint(1, 2),
            ],
        ),
    )
end

@testset "correctly parsing custom consumer signals" begin
    test_json_deser(
        """{
            "name": "consumer1",
            "power": "C1_power",
            "return_temperature": "C1_return_temperature"
        }""",
        CF.ConsumerSignals("consumer1", "C1_power", "C1_return_temperature"),
    )
end

@testset "correctly parsing custom source signals" begin
    test_json_deser(
        """{
            "name": "source1",
            "base_pressure": "S1_base_pressure",
            "pressure_lift": "S1_pressure_lift",
            "temperature": "S1_temperature"
        }""",
        CF.SourceSignals(
            "source1",
            "S1_base_pressure",
            "S1_pressure_lift",
            "S1_temperature",
        ),
    )
end

@testset "correctly parsing custom consumer inputs" begin
    test_json_deser(
        """{
            "input": "input1",
            "factors": {
                "yearly_power_demand": 300,
                "normal_return_temperature": 60
            }
        }""",
        CF.ConsumerInput("input1", CF.ConsumerSignalFactors(300, 60))
    )
end
