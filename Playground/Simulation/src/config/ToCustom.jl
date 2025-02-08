function to_custom(
    (; topology, scenario)::ProprietaryFormat.DistrictHeatingNetwork,
)
    CustomFormat.DistrictHeatingNetwork(
        to_custom(topology),
        to_custom(scenario),
    )
end

function to_custom(
    (; nodes, pipes, consumers, sources)::ProprietaryFormat.Topology,
)
    CustomFormat.Topology(
        to_custom(nodes),
        to_custom(pipes),
        to_custom(consumers),
        to_custom(sources),
    )
end

function to_custom(
    nodes::Dict{String,ProprietaryFormat.Node}
)
    dict_to_vec(
        (name, node) -> begin
            @assert length(node.position) == 3
            position = CustomFormat.Position(
                node.position[1],
                node.position[2],
                node.position[3],
            )
            CustomFormat.Node(name, position, node.is_feed)
        end,
        nodes,
    )
end

function to_custom(
    pipes::Dict{String,ProprietaryFormat.Pipe}
)
    dict_to_vec(
        (name, pipe) -> begin
            @assert length(pipe.nodes) == 2

            CustomFormat.Pipe(
                name,
                pipe.length, pipe.diameter, pipe.transmittance, pipe.roughness, pipe.zeta,
                pipe.nodes[1], pipe.nodes[2],
            )
        end,
        pipes,
    )
end

function to_custom(
    consumers::Dict{String,ProprietaryFormat.Consumer}
)
    dict_to_vec(
        (name, consumer) -> begin
            @assert length(consumer.nodes) == 2
            CustomFormat.Consumer(name, consumer.nodes[1], consumer.nodes[2])
        end,
        consumers,
    )
end

function to_custom(
    sources::Dict{String,ProprietaryFormat.Source}
)
    dict_to_vec(
        (name, source) -> begin
            @assert length(source.nodes) == 2

            CustomFormat.Source(name, source.nodes[1], source.nodes[2])
        end,
        sources,
    )
end

function to_custom(
    (; settings, signals, inputs, consumers, sources, pipes)::ProprietaryFormat.Scenario,
)
    @debug "converting scenario to custom"
    @show inputs

    CustomFormat.Scenario(
        to_custom(settings),
        to_custom(signals),
        Dict(),
        Dict(),
        Dict(),
        #to_custom(consumers),
        #to_custom(sources),
    )
end

function to_custom(
    (;
        feed_temperature, return_temperature, ground_temperature,
        t_start, t_end, dt, ramp, iter, tolerance,
    )::ProprietaryFormat.Settings
)
    CustomFormat.Settings(
        feed_temperature, return_temperature, ground_temperature,
        t_start, t_end, dt, ramp, iter, tolerance,
    )
end

function to_custom(
    signals::Dict{String,ProprietaryFormat.Signal}
)
    Dict(
        Iterators.map(pairs(signals)) do (name, signal)
            function create_poly_signal(degree)
                data = signal.data .|> point -> begin
                    @assert length(point) == 2

                    CustomFormat.DataPoint(point[1], point[2])
                end

                CustomFormat.SignalPoly(
                    name, degree, signal.unit_scale, data,
                )
            end

            name => try
                @match signal.type begin
                    "CONSTANT" => begin
                        CustomFormat.SignalConst(
                            name, signal.unit_scale, signal.data,
                        )
                    end
                    "PIECEWISE_CONSTANT" => create_poly_signal(0)
                    "PIECEWISE_LINEAR" => create_poly_signal(1)
                    "PIECEWISE_QUADRATIC" => create_poly_signal(2)
                    "PIECEWISE_CUBIC" => create_poly_signal(3)
                end
            catch e
                throw(ErrorException("signal type \"" * signal.type * "\" unknown"))
            end
        end
    )
end

function dict_to_vec(f, dict)
    map(collect(dict)) do (key, value)
        f(key, value)
    end
end
