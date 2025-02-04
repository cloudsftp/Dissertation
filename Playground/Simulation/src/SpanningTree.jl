function find_spanning_tree(topology::FeedTopology)
    node_pipes = Dict()
    for node in topology.nodes
        node_pipes[node.name] = Set()
    end

    pipe_indices = Dict()
    for (i, pipe) in enumerate(topology.pipes)
        pipe_indices[pipe.name] = i
        push!(node_pipes[pipe.src], pipe.name)
        push!(node_pipes[pipe.tgt], pipe.name)
    end

    start_node = rand(map(node -> node.name, topology.nodes))

    spanning_tree::Set{String} = Set()
    visited_nodes::Set{String} = Set()
    collect_spanning_tree!(spanning_tree, visited_nodes, start_node, node_pipes, pipe_indices, topology)

    spanning_tree
end

function collect_spanning_tree!(
    spanning_tree::Set{String},
    visited_nodes::Set{String},
    current_node::String,
    node_pipes::Dict{String, String},
    pipe_indices::Dict{String, Int},
    topology::FeedTopology,
)
    push!(current_node, visited_nodes)

    for pipe_name in node_pipes[current_node]
        pipe = topology.pipes[pipe_indices[pipe_name]]
        next_node = Configuration.CustomFormat.get_other_node(pipe, current_node)
        if next_node in visited_nodes
            continue
        end

        push!(spanning_tree, pipe_name)
        collect_spanning_tree!(spanning_tree, visited_nodes, next_node, node_pipes, pipe_indices, topology)
    end
end
