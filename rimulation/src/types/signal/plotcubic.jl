using Plots

f = open("/tmp/cubic")

values = []
for line in readlines(f)
    push!(values, parse(Float64, line))
end

display(plot(values))

sleep(5)
