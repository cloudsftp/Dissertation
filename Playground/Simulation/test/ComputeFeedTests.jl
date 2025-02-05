using Test

using Simulation: compute_feed

const position = CF.Position(0.0, 0.0, 0.0)

include("dummy_network.jl")

function assert_feed_pipes(topology, expected_fee_pipe_names::Vector{String})
    feed = compute_feed(topology)
    feed_pipe_names = Set(map(pipe -> pipe.name, feed.pipes))

    @test feed_pipe_names == Set(expected_fee_pipe_names)
end

@testset "only feed simple" begin
    topology = CF.Topology(
        [
            create_node("F001", true),
            create_node("F002", true),
        ],
        [
            create_pipe("PF001", "F001", "F002"),
        ],
        [],
        [
            create_source("S1", "F001"),
        ]
    )

    assert_feed_pipes(topology, ["PF001"])
end

@testset "only feed small loop" begin
    topology = CF.Topology(
        [
            create_node("F001", true),
            create_node("F002", true),
            create_node("F003", true),
        ],
        [
            create_pipe("PF001", "F001", "F002"),
            create_pipe("PF002", "F001", "F003"),
            create_pipe("PF003", "F002", "F003"),
        ],
        [],
        [
            create_source("S1", "F001"),
        ]
    )

    assert_feed_pipes(topology, ["PF001", "PF002", "PF003"])
end

@testset "small loop" begin
    topology = CF.Topology(
        [
            create_node("F001", true),
            create_node("F002", true),
            create_node("F003", true),
            create_node("R001", true),
            create_node("R002", true),
            create_node("R003", true),
        ],
        [
            create_pipe("PF001", "F001", "F002"),
            create_pipe("PF002", "F001", "F003"),
            create_pipe("PF003", "F002", "F003"),
            create_pipe("PR001", "R001", "R002"),
            create_pipe("PR002", "R001", "R003"),
            create_pipe("PR003", "R002", "R003"),
        ],
        [
            create_consumer("C1", "F003")
        ],
        [
            create_source("S1", "F001"),
        ]
    )

    assert_feed_pipes(topology, ["PF001", "PF002", "PF003"])
end

@testset "only feed disconnected loops" begin
    topology = CF.Topology(
        [
            create_node("F001", true),
            create_node("F002", true),
            create_node("F003", true),
            create_node("F004", true),
            create_node("F005", true),
            create_node("F006", true),
        ],
        [
            create_pipe("PF001", "F001", "F002"),
            create_pipe("PF002", "F001", "F003"),
            create_pipe("PF003", "F002", "F003"),
            create_pipe("PF004", "F004", "F005"),
            create_pipe("PF005", "F004", "F006"),
            create_pipe("PF006", "F005", "F006"),
        ],
        [],
        [
            create_source("S1", "F001"),
        ]
    )

    assert_feed_pipes(topology, ["PF001", "PF002", "PF003"])
end

@testset "only feed connected at one point" begin
    topology = CF.Topology(
        [
            create_node("F001", true),
            create_node("F002", true),
            create_node("F003", true),
            create_node("F004", true),
            create_node("F005", true),
            create_node("F006", true),
        ],
        [
            create_pipe("PF001", "F001", "F002"),
            create_pipe("PF002", "F001", "F003"),
            create_pipe("PF003", "F002", "F004"),
            create_pipe("PF004", "F003", "F004"),
            create_pipe("PF005", "F004", "F005"),
            create_pipe("PF006", "F004", "F006"),
            create_pipe("PF007", "F005", "F006"),
        ],
        [],
        [
            create_source("S1", "F001"),
        ]
    )

    assert_feed_pipes(topology, ["PF001", "PF002", "PF003", "PF004", "PF005", "PF006", "PF007"])
end

@testset "not reaching all source nodes errors" begin
    topology = CF.Topology(
        [
            create_node("F001", true),
            create_node("F002", true),
        ],
        [],
        [],
        [
            create_source("S1", "F001"),
            create_source("S2", "F002"),
        ]
    )

    @test_throws "source nodes not visited:" compute_feed(topology)
end

@testset "not reaching all consumer nodes errors" begin
    topology = CF.Topology(
        [
            create_node("F001", true),
            create_node("F002", true),
        ],
        [],
        [
            create_consumer("C1", "F001"),
            create_consumer("C2", "F002"),
        ],
        [
            create_source("S1", "F001"),
        ]
    )

    @test_throws "consumer nodes not visited:" compute_feed(topology)
end

@testset "not having a source errors" begin
    topology = CF.Topology(
        [
            create_node("F001", true),
            create_node("F002", true),
        ],
        [],
        [
            create_consumer("C1", "F001"),
            create_consumer("C2", "F002"),
        ],
        []
    )

    @test_throws "no sources present" compute_feed(topology)
end
