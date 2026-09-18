-- Gateway: decisions and encounter progression for the shared executor.
-- Raw native identifiers and available controls are in gateway/bindings.h.
local recess_trigger = condition("recess.trigger", any_of("recess.reinforce_trigger", "population.cleared.1"))
local shelf_trigger = condition("shelf.trigger", any_of("shelf.reinforce_trigger", "population.cleared.3", "end.entered"))
local end_trigger = condition("end.trigger", any_of("end.reinforce_trigger", "population.cleared.5"))
local shelf_passed = condition("shelf.route", any_of("end.entered", all_of("population.cleared.3", "population.cleared.4")))
local return_contact = condition("return.engaged", any_of("return.contact", "population.contact.9"))
local vance_ready = condition("vance.ready", all_of("vance.approached", "vance.turned"))
local preroll_done = condition("vance.preroll", all_of("dialogue.finished.9", "dialogue.finished.10"))


local composition = graph("composition", "The Gateway", {
    step("opening", parallel("opening.module", "opening.checked")),
})

local opening = graph("opening", "Gateway traversal and Forest gate", {
    -- Runtime arrival starts presentation without requiring player movement.
    step("presentation", parallel("objective.find_gateway", "dialogue.ikora_gateway")),
    step("recess", "recess.entered", {after={"presentation"}}),
    step("marchers", "marchers.start"),
    step("recess_initial", "recess.initial", {after={"recess", "marchers"}}),
    step("recess_reinforce_trigger", command("recess.trigger", {id="recess.reinforce_trigger"}), {after={"recess_initial"}}),
    step("recess_reinforcements", "recess.reinforcements", {after={"recess_reinforce_trigger"}}),
    step("recess_clear", parallel("recess.cleared", "recess.reinforcements_cleared"), {after={"recess_reinforcements"}}),
    step("recess_cannons", "recess.cannons", {after={"recess_clear"}}),
    -- Both later platform placements are requested as the first platform clears.
    step("shelf_initial", "shelf.initial", {after={"recess_clear"}}),
    step("shelf_reinforce_trigger", command("shelf.trigger", {id="shelf.reinforce_trigger"}), {after={"shelf_initial"}}),
    step("shelf_reinforcements", "shelf.reinforcements", {after={"shelf_reinforce_trigger"}}),
    step("shelf_clear", command("shelf.route", {id="shelf.passed"}), {after={"shelf_reinforcements"}}),
    step("end_initial", "end.initial", {after={"recess_clear"}}),
    step("end_reinforce_trigger", command("end.trigger", {id="end.reinforce_trigger"}), {after={"end_initial"}}),
    step("end_reinforcements", "end.reinforcements", {after={"end_reinforce_trigger"}}),
    step("end_clear", parallel("end.cleared", "end.reinforcements_cleared"), {after={"end_reinforcements"}}),
    step("final_cannon", parallel("end.cannon", "dialogue.vance"), {after={"end_clear"}}),
    step("lighthouse_landing", "lighthouse.landed", {after={"final_cannon"}}),
    -- Place the Forest approach defense when the final platform wave dies.
    step("mainland", parallel("mainland.outskirts", "mainland.center"), {after={"end_clear"}}),
    step("forest_objective", "objective.forest_gate", {after={"lighthouse_landing"}}),
    step("battle_intro", "mainland.intro", {after={"forest_objective"}}),
    step("forest_description", "dialogue.forest_gate", {after={"battle_intro"}}),
    step("mainland_clear", "forest.hydra_dead", {after={"mainland"}}),
    -- X alone owns this cue: Y/Z, trigger volumes and defender counts do not.
    step("forest_approach", "forest.x266"),
    step("forest_ready", "dialogue.at_gate", {after={"forest_approach"}}),
    -- The first barrier contact is latched independently of the defender clear.
    step("forest_attempt", "forest.blocked", {after={"forest_ready"}}),
    step("forest_blocked", parallel("dialogue.blocked", command("vance.return_cue", {id="return.exchange_cue", argument=8960})), {after={"forest_attempt"}}),
    step("bring_sagira", "objective.bring_sagira", {after={"forest_blocked"}}),
})

local ending = graph("ending", "Gateway return, Lighthouse defense and Brother Vance", {
    step("return", parallel("travel.reset", "return.center", "return.outskirts", "objective.return")),
    step("contact", command("return.engaged", {id="return.contact"}), {after={"return"}}),
    step("descendants", "dialogue.descendants", {after={"contact"}}),
    step("wave_one", "finale.wave_1", {after={"return"}}),
    step("wave_two", "finale.wave_2", {after={"wave_one"}}),
    step("wave_three", parallel("finale.wave_3", "finale.support"), {after={"wave_two"}}),
    step("module_exposed", parallel("module.expose", "objective.module", "dialogue.module"), {after={"wave_three"}}),
    step("module_destroyed", "module.destroyed", {after={"module_exposed"}}),
    step("unlocked", parallel("lighthouse.unlock", "objective.enter", "dialogue.timelines"), {after={"module_destroyed"}}),
    step("entered", "lighthouse.entered", {after={"unlocked"}}),
    step("invitation", parallel("dialogue.come_closer", "objective.vance"), {after={"entered"}}),
    step("vance", command("vance.ready", {id="vance.approached"}), {after={"entered"}}),
    step("entry_ghost", "dialogue.old_place", {after={"invitation"}}),
    step("scene", "vance.scene", {after={"preroll", "greeting"}}),
    step("ascent_cue", command("vance.ascent_cue", {id="ascent.clock", argument=22640}), {after={"scene"}}),
    step("ascent", parallel("lighthouse.raise", "lighthouse.light"), {after={"ascent_cue"}}),
    step("ending_cue", command("vance.ending_cue", {id="ending.clock", argument=31000}), {after={"scene"}}),
    step("complete", "mission.finish", {after={"ascent", "ending_cue", "descendants"}}),
    step("greeting", "vance.greeting", {after={"invitation"}}),
    step("preroll", "vance.preroll", {after={"vance", "entry_ghost"}}),
})

return mission{
    id="gateway", graphs={composition, opening, ending},
    roles={mission="composition", opening="opening", ending="ending"},
    entry="composition", modules={"opening"}, observations={"opening.checked"},
    phases={"opening", "ending"},
    conditions={recess_trigger, shelf_trigger, end_trigger, shelf_passed, return_contact, vance_ready, preroll_done},
    presentation=presentation{markers={
        {objective="0xF7A5BB63", target="forest_gate"},
        {objective="0xFA35EDEE", target="lighthouse_portal"},
        {objective="0xB4D880CB", target="lighthouse_portal"},
        {objective="0x80FD8B67", target="module"},
        {objective="0xB10D6455", target="lighthouse_portal"},
        {objective="0x722FE621", target="vance"},
    }},
}
