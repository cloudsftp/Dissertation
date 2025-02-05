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
