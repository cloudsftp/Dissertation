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
    @test tree == expected
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
        ["PF001", "PF003"],
    )
end
