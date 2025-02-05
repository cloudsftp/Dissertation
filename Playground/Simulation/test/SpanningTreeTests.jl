using Test

using Simulation: find_spanning_tree

include("dummy_network.jl")

@testset "malformed feed topology" begin
    @test_throws "feed network does not have any nodes" find_spanning_tree(
        Simulation.FeedTopology(
            [], [], [], [],
        )
    )
    @test_throws "is connected to non-existant node" find_spanning_tree(
        Simulation.FeedTopology(
            [],
            [
                create_pipe("PF001", "A", "B"),
            ],
            [],
            [],
        )
    )
end

function test_spanning_tree(
    nodes::Vector{String},
    pipes::Vector{CF.Pipe},
    expected::Vector{String},
)
    nodes = map(create_node, nodes)

    feed = Simulation.FeedTopology(
        nodes,
        pipes,
        [],
        [],
    )

    tree = find_spanning_tree(feed)
    @test tree == Set(expected)
end

@testset "single pipe" begin
    test_spanning_tree(
        ["F001", "F002"],
        [
            create_pipe("PF001", "F001", "F002"),
        ],
        ["PF001"],
    )
end

@testset "two pipes" begin
    test_spanning_tree(
        ["F001", "F002", "F003"],
        [
            create_pipe("PF001", "F001", "F002"),
            create_pipe("PF002", "F001", "F003"),
        ],
        ["PF001", "PF002"],
    )
end

@testset "small cycle" begin
    test_spanning_tree(
        ["F001", "F002", "F003"],
        [
            create_pipe("PF001", "F001", "F002"),
            create_pipe("PF002", "F001", "F003"),
            create_pipe("PF003", "F002", "F003"),
        ],
        ["PF001", "PF002"],
    )
end

@testset "two cycles" begin
    test_spanning_tree(
        map(string, 1:8),
        [
            create_pipe("P1", "1", "2"),
            create_pipe("P2", "2", "3"),
            create_pipe("P3", "3", "4"),
            create_pipe("P4", "3", "5"),
            create_pipe("P5", "4", "6"),
            create_pipe("P6", "5", "6"),
            create_pipe("P7", "6", "7"),
            create_pipe("P8", "7", "8"),
            create_pipe("P9", "8", "2"),
        ],
        map(n -> "P" * string(n), [1, 2, 3, 4, 5, 8, 9]),
    )
end
