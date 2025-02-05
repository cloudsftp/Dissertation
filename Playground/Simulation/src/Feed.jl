struct Pipe
    # TODO: custom type for local usage
end

struct Node
    # TODO: custom type for local usage
end

struct Consumer
    name::String
    node::String
end

struct Source
    name::String
    node::String
end

struct FeedTopology
    nodes::Vector{Configuration.CustomFormat.Node}
    pipes::Vector{Configuration.CustomFormat.Pipe}
    sources::Vector{Source}
    consumers::Vector{Consumer}
end

function compute_feed(topology::Configuration.CustomFormat.Topology)
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

    source_nodes = Set()
    for source in topology.sources
        push!(source_nodes, source.tgt)
    end

    if length(source_nodes) == 0
        throw(ErrorException("no sources present"))
    end

    start_node = rand(source_nodes)
    feed_pipe_names::Set{String} = Set()
    visited_nodes::Set{String} = Set()
    collect_feed!(feed_pipe_names, visited_nodes, start_node, node_pipes, pipe_indices, topology)

    if !(source_nodes ⊆ visited_nodes)
        throw(ErrorException("source nodes not visited: " * string(setdiff(source_nodes, visited_nodes))))
    end

    consumer_nodes = Set()
    for consumer in topology.consumers
        push!(consumer_nodes, consumer.src)
    end

    if !(consumer_nodes ⊆ visited_nodes)
        throw(ErrorException("consumer nodes not visited: " * string(setdiff(consumer_nodes, visited_nodes))))
    end

    build_feed_topology(feed_pipe_names, visited_nodes, topology)
end

function collect_feed!(feed_pipe_names, visited_nodes, current_node, node_pipes, pipe_indices, topology)
    for pipe_name in node_pipes[current_node]
        if pipe_name in feed_pipe_names
            continue
        end

        push!(feed_pipe_names, pipe_name)

        pipe = topology.pipes[pipe_indices[pipe_name]]
        next_node = Configuration.CustomFormat.get_other_node(pipe, current_node)
        collect_feed!(feed_pipe_names, visited_nodes, next_node, node_pipes, pipe_indices, topology)
    end

    push!(visited_nodes, current_node)
end

function build_feed_topology(feed_pipe_names::Set{String}, visited_nodes::Set{String}, topology)
    nodes = []
    for node in topology.nodes
        if node.name in visited_nodes
            push!(nodes, node)
        end
    end

    pipes = []
    for pipe in topology.pipes
        if pipe.name in feed_pipe_names
            push!(pipes, pipe)
        end
    end

    sources = []
    for source in topology.sources
        push!(sources, Source(source.name, source.tgt))
    end

    consumers = []
    for consumer in topology.consumers
        push!(consumers, Consumer(consumer.name, consumer.src))
    end

    FeedTopology(nodes, pipes, sources, consumers)
end
