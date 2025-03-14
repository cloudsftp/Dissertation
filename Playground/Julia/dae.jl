using OrdinaryDiffEq, Plots

function lorenz!(du, u, _, _)
 du[1] = 10.0 * (u[2] - u[1])
 du[2] = u[1] * (28.0 - u[3]) - u[2]
 du[3] = u[1] * u[2] - (8 / 3) * u[3]
end

u0 = [1.0; 0.0; 0.0]
tspan = (0.0, 100.0)
prob = ODEProblem(lorenz!, u0, tspan)

sol = solve(prob, Tsit5())
result = plot(sol, vars=(1,2,3))

savefig(result, "result.png" )
