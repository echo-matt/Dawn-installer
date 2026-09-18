-- Native bindings supply identities and authentic observations. Lua owns the
-- traversal fallbacks, encounter gates, phases, and completion.
local overpass = condition("overpass.arrived", any_of("overpass.entered",
    "overpass.dropship", "overpass.fallback", "overpass.directive", "overpass.waypoint"))
local choke = condition("choke.arrived", any_of("choke.entered", "overpass.arrived"))
local streets = condition("streets.arrived", any_of("streets.entered", "choke.arrived"))
local followers = condition("followers.arrived", any_of("followers.entered", "pike.mounted", "streets.arrived"))
local tower = condition("tower.arrived", any_of("tower.entered", "tower.directive", "tower.waypoint"))
local tunnel = condition("tunnel.arrived", any_of("tunnel.entered", "tower.arrived"))
local cliff = condition("cliff.arrived", any_of("cliff.entered", "tunnel.arrived"))
local trial = condition("trial.arrived", any_of("trial.entered", "trial.directive", "cliff.arrived"))
local revive = condition("revive.ready", all_of("tower.cleared", "lair.cleared"))
local revival = condition("revival.ready", all_of("revive.accepted", "prelude.finished"))
local finish = condition("finish.ready", all_of("revive.scene_finished", "revive.audio_finished"))

local composition = graph("composition", "A Deadly Trial", sequence(
    step("mission", parallel("opening.module", "opening.checked"))
))

local opening = graph("opening", "A Deadly Trial opening", {
    -- The runtime starts this graph on confirmed mission arrival.
    step("coordinates", parallel("objective.coordinates", "dialogue.0")),
    step("square", "square.entered", {after={"coordinates"}}),
    step("square_fallen", "square.fallen", {after={"square"}}),
    step("pikes", parallel("objective.pike", "square.pikes"), {after={"square_fallen"}}),
    step("followers", "followers.arrived", {after={"pikes"}}),
    step("faith", parallel("dialogue.1", "objective.path"), {after={"followers"}}),
    step("streets", "streets.arrived", {after={"faith"}}),
    step("street_resistance", "streets.fallen", {after={"streets"}}),
    step("choke", "choke.arrived", {after={"street_resistance"}}),
    step("choke_resistance", "choke.fallen", {after={"choke"}}),
    step("overpass", "overpass.arrived", {after={"choke_resistance"}}),
    step("roadblock", parallel("objective.walker", "overpass.support", "walker.enable"), {after={"overpass"}}),
    -- The independent death wait releases tower intent in the death callback's
    -- generic event pump, even while roadblock dialogue is unsubmitted.
    step("walker_death", "walker.cleared", {after={"overpass"}}),
    step("tower_enable", "tower.enable", {after={"walker_death"}}),
    step("tower_clear", command("tower.cleared", {id="tower.deaths_for_lair"}), {after={"tower_enable"}}),
    step("lair_prepare", "lair.enable", {after={"tower_clear"}}),
    step("barrier", parallel("barrier.open", "objective.temple", "overpass.pikes"), {after={"roadblock", "walker_death"}}),
    step("trial", "trial.arrived", {after={"barrier"}}),
    step("survival", "dialogue.2", {after={"trial"}}),
    step("cliff", "cliff.arrived", {after={"survival"}}),
    step("cliff_resistance", "cliff.fallen", {after={"cliff"}}),
    step("tunnel", "tunnel.arrived", {after={"cliff_resistance"}}),
    step("tunnel_resistance", "tunnel.fallen", {after={"tunnel"}}),
    step("tower", "tower.arrived", {after={"tunnel_resistance"}}),
    step("tower_resistance", parallel("dialogue.4", "tower.cleared"), {after={"tower", "tower_enable"}}),
    step("search", "objective.search", {after={"tower_resistance"}})
})

local ending = graph("ending", "A Deadly Trial ending", sequence(
    step("lair", "lair.entered"),
    step("ambush", "lair.marauders"),
    step("bodies", "bodies.entered"),
    step("revive_gate", "revive.ready"),
    step("followers_dead", parallel("dialogue.7", "objective.revive", "revive.enable")),
    step("interact_and_prelude", "revival.ready"),
    -- Native scene and audio ownership retain authentic receipt validation.
    step("revival", "revive.scene"),
    step("revival_finished", "finish.ready"),
    step("finish", "mission.finish")
))

return mission{
    id = "deadly_trial",
    graphs = {composition, opening, ending},
    roles = {mission="composition", opening="opening", ending="ending"},
    phases = {"opening", "ending"},
    conditions = {overpass, choke, streets, followers, tower, tunnel, cliff, trial, revive, revival, finish},
    entry = "composition",
    modules = {"opening"},
    observations = {"opening.checked"},
}
