const PF = Simulation.Configuration.ProprietaryFormat
const CF = Simulation.Configuration.CustomFormat

using Simulation.Configuration: to_custom

function assert_equals_unordered(actual::Vector, expected::Vector)
    @test length(expected) == length(actual)

    @test all(
        expected_element -> begin
            count_expected = count(
                element -> expected_element == element,
                expected,
            )
            count_actual = count(
                element -> expected_element == element,
                actual,
            )

            count_expected == count_actual
        end,
        expected,
    )
end

@testset "unordered vector compare" begin
    assert_equals_unordered([1, 2, 3], [1, 2, 3])
    assert_equals_unordered([1, 2, 3], [1, 3, 2])
    assert_equals_unordered([1, 2, 2, 3], [1, 2, 3, 2])
end

function assert_equals_unordered(actual::CF.Topology, expected::CF.Topology)
    assert_equals_unordered(expected.nodes, actual.nodes)
    assert_equals_unordered(expected.pipes, actual.pipes)
    assert_equals_unordered(expected.consumers, actual.consumers)
    assert_equals_unordered(expected.sources, actual.sources)
end

@testset "convert topology to custom format"  begin
    proprietary_topology = PF.Topology(
        Dict(
            "VL001" => PF.Node([1.0, 1.0, 3.0], true),
            "VL002" => PF.Node([1.0, 2.0, 3.0], true),
            "VL003" => PF.Node([2.0, 1.0, 3.0], true),
            "VL004" => PF.Node([2.0, 2.0, 3.0], true),
        ),
        Dict(
            "PF001" => PF.Pipe(
                ["VL001", "VL002"],
                1.2, 1, 1, 1, 1,
            ),
            "PF002" => PF.Pipe(
                ["VL001", "VL003"],
                1.2, 1, 1, 1, 1,
            ),
            "PF003" => PF.Pipe(
                ["VL002", "VL004"],
                1.2, 1, 1, 1, 1,
            ),
            "PF004" => PF.Pipe(
                ["VL003", "VL004"],
                1.2, 1, 1, 1, 1,
            ),
        ),
        Dict(
            "C1" => PF.Consumer(["VL004", "VL001"]),
        ),
        Dict(
            "S1" => PF.Source(["VL004", "VL001"]),
        )
    )
    actual = to_custom(proprietary_topology)

    expected = CF.Topology(
        [
            CF.Node("VL001", CF.Position(1.0, 1.0, 3.0), true),
            CF.Node("VL002", CF.Position(1.0, 2.0, 3.0), true),
            CF.Node("VL003", CF.Position(2.0, 1.0, 3.0), true),
            CF.Node("VL004", CF.Position(2.0, 2.0, 3.0), true),
        ],
        [
            CF.Pipe("PF001", 1.2, 1, 1, 1, 1, "VL001", "VL002"),
            CF.Pipe("PF002", 1.2, 1, 1, 1, 1, "VL001", "VL003"),
            CF.Pipe("PF003", 1.2, 1, 1, 1, 1, "VL002", "VL004"),
            CF.Pipe("PF004", 1.2, 1, 1, 1, 1, "VL003", "VL004"),
        ],
        [CF.Consumer("C1", "VL004", "VL001")],
        [CF.Source("S1", "VL004", "VL001")],
    )

    assert_equals_unordered(actual, expected)
end

function assert_equals_unordered(actual::CF.Scenario, expected::CF.Scenario)
    @test expected.settings == actual.settings

    assert_equals_unordered(expected.signals, actual.signals)
    assert_equals_unordered(expected.inputs, actual.inputs)
    assert_equals_unordered(expected.consumers, actual.consumers)
    assert_equals_unordered(expected.sources, actual.sources)
    assert_equals_unordered(expected.pipes, actual.pipes)
end

@testset "convert topology to custom format"  begin
    proprietary_scenario = PF.Scenario(
        PF.Settings(
            100, 60, 10,
            0, 4.8, 15, 8,
            1000, 1e-6,
        ),
        Dict(
            "one" => PF.Signal(
                "CONSTANT",
                [["time", "min"], ["pressure", "Pa"]],
                1, 1.,
            ),
            "signal1" => PF.Signal(
                "PIECEWISE_CUBIC",
                [["time", "min"], ["pressure", "Pa"]],
                1,
                [
                    [0., 1.],
                    [5., 5.],
                    [10., 3.],
                ]
            ),
        ),
        Dict(
            "input1" => PF.Input(["one", "signal1"]),
        ),
        Dict(
            "C1" => PF.ConsumerSignal(60, 2, "input1"),
        ),
        Dict(
            "S1" => PF.SourceSignal("Source2", "input1"),
        ),
        Dict(
            "PF001" => PF.PipeSignal("input1"),
            "PF002" => PF.PipeSignal("input1"),
            "PF003" => PF.PipeSignal("input1"),
            "PF004" => PF.PipeSignal("input1"),
        ),
    )
    actual = to_custom(proprietary_scenario)

    expected = CF.Scenario(
        CF.Settings(
            100, 60, 10,
            0, 4.8, 15, 8,
            1000, 1e-6,
        ),
        [
            CF.SignalConst(
                "one", 1, 1.,
            ),
            CF.SignalPoly(
                "signal1", 3, 1,
                [
                    CF.DataPoint(0., 1.),
                    CF.DataPoint(5., 5.),
                    CF.DataPoint(10., 3.),
                ]
            ),
        ],
        [
            CF.Input("input1", ["one", "signal1"]),
        ],
        [
            CF.ConsumerInputMapping(
                "C1", "input1", 60, 2,
            ),
        ],
        [
            CF.InputMapping("S1", "input1"),
        ],
        [
            CF.InputMapping("PF001", "input1"),
            CF.InputMapping("PF002", "input1"),
            CF.InputMapping("PF003", "input1"),
            CF.InputMapping("PF004", "input1"),
        ]
    )

    assert_equals_unordered(actual, expected)
end

@testset "quick check" begin
    signals = Dict(
        "signal1" => PF.Signal(
            "CUBIC",
            [["time", "min"], ["pressure", "Pa"]],
            1,
            [
                [0., 1.],
                [5., 5.],
                [10., 3.],
            ]
        ),
    )

    @test_throws "signal type \"CUBIC\" unknown" to_custom(signals)
end
