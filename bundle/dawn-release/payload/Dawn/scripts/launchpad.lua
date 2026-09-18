-- New Light. The registered C++ module owns encounters, cinematics,
-- native receipts and run ownership. Lua supplies the server entry point.
local composition = graph("composition", "New Light", sequence(
    step("mission", parallel("mission.native", "mission.finished"))
))

return mission{
    id = "launchpad",
    graphs = {composition},
    roles = {mission = "composition"},
    entry = "composition",
    modules = {"native"},
    observations = {"mission.finished"},
}
