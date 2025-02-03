const PF = Simulation.Configuration.ProprietaryFormat

include("json.jl")

@testset "no exception when parsing proprietary configuration example" begin
    base_path = "../Data/Proprietary Format"
    @test try
        PF.load(base_path)
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
        PF.Configuration("a", "b"),
    )
end

@testset "correctly parsing proprietary node" begin
    test_json_deser(
        """{
            "position": [100.0, 2, 3],
            "is_feed": true
        }""",
        PF.Node([100, 2, 3], true),
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
        PF.Pipe(["A", "B"], 1, 2, 3, 4, 5),
    )
end

@testset "correctly parsing proprietary consumer" begin
    test_json_deser(
        """{
            "nodes": ["A", "B"]
        }""",
        PF.Consumer(["A", "B"]),
    )
end

@testset "correctly parsing proprietary source" begin
    test_json_deser(
        """{
            "nodes": ["A", "B"]
        }""",
        PF.Source(["A", "B"]),
    )
end

@testset "correctly parsing proprietary settings" begin
    test_json_deser(
        """{
            "feed_temperature [C]": 120,
            "return_temperature [C]": 60.,
            "ground_temperature [C]": 1e0,
            "t_start [d]": 0,
            "t_end [d]": 2.4,
            "dt [min]": 15,
            "ramp [h]": 8,
            "num_iter": 1000,
            "tol": 1e-12
        }""",
        PF.Settings(
            120, 60, 1,
            0, 2.4, 15, 8,
            1000, 1e-12,
        ),
    )
end

@testset "correctly parsing proprietary signal" begin
    test_json_deser(
        """{
            "type": "CONSTANT",
            "axes": [["time", "min"], ["pressure", "Pa"]],
            "unit_scale": 10e6,
            "data": 1
        }""",
        PF.Signal(
            "CONSTANT",
            [["time", "min"], ["pressure", "Pa"]],
            10e6, 1.,
        ),
    )

    test_json_deser(
        """{
            "type": "PIECEWISE_CUBIC",
            "axes": [["time", "min"], ["pressure", "Pa"]],
            "unit_scale": 10e6,
            "data": [[0, 1], [1, 2], [3, -1]]
        }""",
        PF.Signal(
            "PIECEWISE_CUBIC",
            [["time", "min"], ["pressure", "Pa"]],
            10e6, [[0., 1.], [1., 2.], [3., -1.]],
        ),
    )
end

@testset "correctly parsing proprietary input" begin
    test_json_deser(
        """{
            "signals": ["A", "B"]
        }""",
        PF.Input(["A", "B"]),
    )
end

@testset "correctly parsing proprietary consumer signal" begin
    test_json_deser(
        """{
            "return_temperature": 60,
            "annual_consumption": 2,
            "input": "input1"
        }""",
        PF.ConsumerSignal(60, 2, "input1"),
    )
end

@testset "correctly parsing proprietary source signal" begin
    test_json_deser(
        """{
            "type": "Source2",
            "input": "input1"
        }""",
        PF.SourceSignal("Source2", "input1"),
    )
end

@testset "correctly parsing proprietary pipe signal" begin
    test_json_deser(
        """{
            "input": "input1"
        }""",
        PF.PipeSignal("input1"),
    )
end
