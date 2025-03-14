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

@testset "compare unordered vectors" begin
    assert_equals_unordered([1, 2, 3], [1, 2, 3])
    assert_equals_unordered([1, 2, 3], [1, 3, 2])
    assert_equals_unordered([1, 2, 2, 3], [1, 2, 3, 2])
end

function test_to_custom(proprietary, custom)
    actual = to_custom(proprietary)
    if actual isa Vector && custom isa Vector
        assert_equals_unordered(actual, custom)
    else
        @test actual == custom
    end
end

@testset "convert nodes to custom format" begin
    test_to_custom(
        Dict(
            "VL001" => PF.Node([1.0, 1.0, 3.0], true),
            "VL002" => PF.Node([1.0, 2.0, 3.0], true),
            "VL003" => PF.Node([2.0, 1.0, 3.0], true),
            "VL004" => PF.Node([2.0, 2.0, 3.0], true),
        ),
        [
            CF.Node("VL001", CF.Position(1.0, 1.0, 3.0), true),
            CF.Node("VL002", CF.Position(1.0, 2.0, 3.0), true),
            CF.Node("VL003", CF.Position(2.0, 1.0, 3.0), true),
            CF.Node("VL004", CF.Position(2.0, 2.0, 3.0), true),
        ],
    )
end

@testset "convert pipes to custom format" begin
    test_to_custom(
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
        [
            CF.Pipe("PF001", 1.2, 1, 1, 1, 1, "VL001", "VL002"),
            CF.Pipe("PF002", 1.2, 1, 1, 1, 1, "VL001", "VL003"),
            CF.Pipe("PF003", 1.2, 1, 1, 1, 1, "VL002", "VL004"),
            CF.Pipe("PF004", 1.2, 1, 1, 1, 1, "VL003", "VL004"),
        ],
    )
end

@testset "convert consumers to custom form" begin
    test_to_custom(
        Dict(
            "C1" => PF.Consumer(["VL004", "VL001"]),
            "C2" => PF.Consumer(["VL014", "VL101"]),
            "C3" => PF.Consumer(["VL004", "VL051"]),
        ),
        [
            CF.Consumer("C1", "VL004", "VL001"),
            CF.Consumer("C2", "VL014", "VL101"),
            CF.Consumer("C3", "VL004", "VL051"),
        ],
    )
end

@testset "convert sources to custom format"  begin
    test_to_custom(
        Dict(
            "S1" => PF.Source(["VL004", "VL001"]),
            "S2" => PF.Source(["VL034", "VL201"]),
            "S3" => PF.Source(["VL004", "VL031"]),
        ),
        [
            CF.Source("S1", "VL004", "VL001"),
            CF.Source("S2", "VL034", "VL201"),
            CF.Source("S3", "VL004", "VL031"),
        ],
    )
end

@testset "convert settings to custom format"  begin
    test_to_custom(
        PF.Settings(
            100, 60, 10,
            0, 4.8, 15, 8,
            1000, 1e-6,
        ),
        CF.Settings(
            100, 60, 10,
            0, 4.8, 15, 8,
            1000, 1e-6,
        ),
    )
end

@testset "convert signals to custom format"  begin
    test_to_custom(
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
            "one" => CF.SignalConst(
                "one", 1, 1.,
            ),
            "signal1" => CF.SignalPoly(
                "signal1", 3, 1,
                [
                    CF.DataPoint(0., 1.),
                    CF.DataPoint(5., 5.),
                    CF.DataPoint(10., 3.),
                ]
            ),
        ),
    )
end

@testset "convert signals to custom format throws exception when type is unknown" begin
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

const zero_settings = PF.Settings(0, 0, 0, 0, 0, 0, 0, 0, 0)

@testset "convert consumer signals and inputs to custom format"  begin
    proprietary_scenario = PF.Scenario(
        zero_settings,
        Dict(
            "power" => PF.Signal(
                "CONSTANT",
                [["t", "min"], ["W", "J"]],
                1e9, 8.,
            ),
            "temperature" => PF.Signal(
                "PIECEWISE_CUBIC",
                [["t", "min"], ["1", "1"]],
                1,
                [[0., 1.], [5., 0.9], [10., 1.1]],
            ),
        ),
        Dict(
            "CON" => PF.Input(["power", "temperature"])
        ),
        Dict(
            "C1" => PF.ConsumerSignal(60, 300, "CON"),
        ),
        Dict(),
        Dict(),
    )

    custom_scenario = to_custom(proprietary_scenario)

    @test custom_scenario.signals == Dict(
        "power" => CF.SignalConst("power", 1e9, 8),
        "temperature" => CF.SignalPoly("temperature", 3, 1, [
            CF.DataPoint(0, 1),
            CF.DataPoint(5, 0.9),
            CF.DataPoint(10, 1.1),
        ]),
    )
    @test custom_scenario.inputs == Dict(
        "CON" => CF.ConsumerSignals("CON", "power", "temperature")
    )
    @test custom_scenario.consumer_inputs == Dict(
        "C1" => CF.ConsumerInput("CON", CF.ConsumerSignalFactors(300, 60)),
    )
end

@testset "convert source signals and inputs to custom format"  begin
    proprietary_scenario = PF.Scenario(
        zero_settings,
        Dict(
            "base_pressure" => PF.Signal(
                "CONSTANT",
                [["t", "min"], ["p", "Pa"]],
                1e5, 5.,
            ),
            "pressure_lift" => PF.Signal(
                "CONSTANT",
                [["t", "min"], ["p", "Pa"]],
                1e5, 4.5,
            ),
            "temperature" => PF.Signal(
                "PIECEWISE_CUBIC",
                [["t", "min"], ["T", "C"]],
                1,
                [[0., 120.], [5., 115.0], [10., 105.]],
            ),
        ),
        Dict(
            "SRC" => PF.Input(["base_pressure", "pressure_lift", "temperature"]),
        ),
        Dict(),
        Dict(
            "S1" => PF.SourceSignal("Source2", "SRC"),
        ),
        Dict(),
    )

    custom_scenario = to_custom(proprietary_scenario)

    @test custom_scenario.signals == Dict(
        "base_pressure" => CF.SignalConst("base_pressure", 1e5, 5),
        "pressure_lift" => CF.SignalConst("pressure_lift", 1e5, 4.5),
        "temperature" => CF.SignalPoly("temperature", 3, 1, [
            CF.DataPoint(0, 120),
            CF.DataPoint(5, 115),
            CF.DataPoint(10, 105),
        ]),
    )
    @test custom_scenario.inputs == Dict(
        "SRC" => CF.SourceSignals("SRC", "base_pressure", "pressure_lift", "temperature"),
    )
    @test custom_scenario.source_inputs == Dict(
        "S1" => "SRC",
    )
end
