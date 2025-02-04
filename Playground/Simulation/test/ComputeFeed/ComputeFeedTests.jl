using Test

using Simulation: compute_feed

const position = CF.Position(0., 0., 0.)

function create_node(name::String, feed::Bool)
    CF.Node(name, position, feed)
end

function create_source(name::String, tgt::String)
    CF.Source(name, "", tgt)
end

function create_consumer(name::String, src::String)
    CF.Consumer(name, src, "")
end

function create_pipe(name::String, src::String, tgt::String)
    CF.Pipe(name, 0., 0., 0., 0., 0., src, tgt)
end

@testset "only feed simple" begin
    topology = CF.Topology(
        [
            create_node("F001", true),
            create_node("F002", true),
        ],
        [
            create_pipe("PF001", "F001", "F001"),
        ],
        [],
        [
            create_source("S1", "F001"),
        ]
    )

    feed = compute_feed(topology)

    feed_pipe_names = Set(map(pipe -> pipe.name, feed.pipes))
    @test feed_pipe_names == Set(["PF001"])
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
