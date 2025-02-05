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
    CF.Pipe(name, 0.0, 0.0, 0.0, 0.0, 0.0, src, tgt)
end
