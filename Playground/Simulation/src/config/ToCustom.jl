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
    CustomFormat.Scenario(
        to_custom(settings),
        to_custom(signals),
        to_custom(inputs),
        to_custom(consumers),
        to_custom(sources),
        to_custom(pipes),
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
    dict_to_vec(
        (name, signal) -> begin
            create_poly_signal(degree) = begin
                data = signal.data .|> point -> begin
                    @assert length(point) == 2

                    CustomFormat.DataPoint(point[1], point[2])
                end

                CustomFormat.SignalPoly(name, degree, signal.unit_scale, data)
            end

            @match signal.type begin
                "CONSTANT" => begin
                    CustomFormat.SignalConst(name, signal.unit_scale, signal.data)
                end
                "PIECEWISE_CONSTANT" => create_poly_signal(0)
                "PIECEWISE_LINEAR" => create_poly_signal(1)
                "PIECEWISE_QUADRATIC" => create_poly_signal(2)
                "PIECEWISE_CUBIC" => create_poly_signal(3)
            end
        end,
        signals,
    )
end

function to_custom(
    inputs::Dict{String,ProprietaryFormat.Input}
)
    dict_to_vec(
        (name, (; signals)) -> begin
            CustomFormat.Input(name, signals)
        end,
        inputs,
    )
end

function to_custom(
    consumers::Dict{String,ProprietaryFormat.ConsumerSignal}
)
    dict_to_vec(
        (name, (; return_temperature, annual_consumption, input)) -> begin
            CustomFormat.ConsumerInputMapping(
                name, input, return_temperature, annual_consumption,
            )
        end,
        consumers,
    )
end

function to_custom(
    sources::Dict{String,ProprietaryFormat.SourceSignal}
)
    dict_to_vec(
        (name, (; type, input)) -> begin
            CustomFormat.InputMapping(name, input)
        end,
        sources,
    )
end

function to_custom(
    pipes::Dict{String,ProprietaryFormat.PipeSignal}
)
    dict_to_vec(
        (name, (; input)) -> begin
            CustomFormat.InputMapping(name, input)
        end,
        pipes,
    )
end

function dict_to_vec(f, dict)
    map(collect(dict)) do (; first, second)
        f(first, second)
    end
end
