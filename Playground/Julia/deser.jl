using Serde

struct Test
    a::Any
end

function Serde.deser(::Test, ::Union{Int64, Vector{Int64}}, x::Int64)
    x
end

function Serde.deser(::Test, ::Union{Int64, Vector{Int64}}, x::Vector{Int64})
    x::Union{Int64, Vector{Int64}}
end

function main()
    @show deser_json(Test, "{\"a\": 1}")
    @show deser_json(Test, "{\"a\": [1]}")
end

main()
