using .Configuration.CustomFormat: get_other_node

using DataStructures

function find_spanning_tree(feed::FeedTopology)
    node_pipes = Dict{String, Vector{String}}()
    for node in feed.nodes
        node_pipes[node.name] = []
    end

    pipe_indices = Dict{String, Int}()
    for (i, pipe) in enumerate(feed.pipes)
        if !(pipe.src in keys(node_pipes) && pipe.tgt in keys(node_pipes))
            throw(ErrorException("pipe " * string(pipe) * " is connected to non-existant node"))
        end

        pipe_indices[pipe.name] = i
        push!(node_pipes[pipe.src], pipe.name)
        push!(node_pipes[pipe.tgt], pipe.name)
    end

    if length(feed.nodes) == 0
        throw(ErrorException("feed network does not have any nodes"))
    end

    spanning_tree = Set{String}()

    start_node = feed.nodes[1].name
    work = Queue{Tuple{String, String}}()

    function enqueue_work_items!(node::String)
        for pipe in node_pipes[node]
            if pipe in spanning_tree
                continue
            end

            enqueue!(work, (node, pipe))
        end
    end

    enqueue_work_items!(start_node)
    visited_nodes = Set{String}([start_node])

    while !isempty(work)
        (current_node, pipe_name) = dequeue!(work)

        pipe = feed.pipes[pipe_indices[pipe_name]]
        next_node = get_other_node(pipe, current_node)
        if next_node in visited_nodes
            continue
        end

        push!(spanning_tree, pipe_name)
        push!(visited_nodes, next_node)
        enqueue_work_items!(next_node)
    end

    spanning_tree
end
