using .Simulation.Configuration.CustomFormat: SignalConst, SignalPoly, Scenario

const Signal = Union{SignalConst,SignalPoly}

struct ConsumerSignals
    demand::Signal
    #return_temperature::Signal
end

struct SourceSignals
    pressure::Signal
    #base_pressure
    #pressure_lift
    temperature::Signal
end

struct Signals
    consumer_signals::Dict{String,ConsumerSignals}
    source_signals::Dict{String,SourceSignals}
end

function Signals(scenario::Scenario)
    @show scenario

    consumer_signals = ConsumerSignals(demand)

    Signals(consumer_signals, source_signals)
end
