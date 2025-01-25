const PF = Simulation.Configuration.ProprietaryFormat
const CF = Simulation.Configuration.CustomFormat

using Simulation.Configuration: to_custom

function assert_equals_unordered(expected::CF.Topology, actual::CF.Topology)
    assert_equals_unordered(expected.nodes, actual.nodes)
    assert_equals_unordered(expected.pipes, actual.pipes)
    assert_equals_unordered(expected.consumers, actual.consumers)
    assert_equals_unordered(expected.sources, actual.sources)
end

function assert_equals_unordered(expected::Vector, actual::Vector)
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

    assert_equals_unordered(expected, actual)
end
