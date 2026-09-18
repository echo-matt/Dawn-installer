-- Tree of Probabilities. Four regions: the Lighthouse opening, Infinite Forest B, the Chase,
-- and the bomb ledge/Valus Thuun encounter. Native identifiers and the
-- controls each name is allowed to use are in state/activity/strike_pact/bindings.h; the section
-- catalogs beside it hold the identities. Every gate below is the one the proven mission policy
-- used, and each phase opens on its own held region because a portal or a fade delivers the
-- player there rather than a volume they walk into.

local composition = graph("composition", "Tree of Probabilities", {
    step("mission", parallel("opening.module", "opening.checked")),
})

local forestDefenseReached = condition("forest.defense.reached",
    any_of("forest.load_post", "forest.portal.seen", "forest.portal.reached"))
local openingDeparted = condition("opening.departed", any_of("tunnel.entered", "region.forest"))
local forestExitReached = condition("forest.exit.reached",
    any_of("forest.portal.seen", "forest.portal.reached", "region.chase"))
local forestDeparted = condition("forest.departed", any_of("forest.leaving", "region.chase"))
local sparrowReached = condition("chase.sparrow.reached", any_of("chase.sparrow", "chase.jump"))

-- The Lighthouse. Approach the gateway, clear the local defense, take the portal.
local opening = graph("opening", "Lighthouse gateway", {
    step("arrival", parallel("objective.approach", "shield.raise", "opening.stage1")),
    -- Ikora's briefing and the vanguard formation.
    step("briefing", parallel("dialogue.intro", "opening.stage2"), {after={"arrival"}}),
    step("gate", "gate.entered", {after={"briefing"}}),
    -- The Red Legion survivors line accompanies the gateway defense.
    step("defense", parallel("dialogue.gate", "opening.stage3"), {after={"gate"}}),
    -- Rearguards, harassers and the named Gladiator: the local defense, not every enemy on the map.
    step("cleared", "gateway.cleared", {after={"defense"}}),
    step("open", parallel("shield.drop", "objective.traverse", "portal.activate"), {after={"cleared"}}),
    step("departed", "opening.departed", {after={"open"}}),
    -- The beacon is cleared here: the authored portal point is behind the player from now on.
    step("clear_marker", "marker.clear", {after={"departed"}}),
})

-- Infinite Forest B. The generator activation is the section: without it the worker places no
-- encounter at all and every island is empty.
local forest = graph("forest", "Infinite Forest B", {
    step("arrive", "region.forest"),
    step("prepare", parallel("forest.generate", "forest.shield.raise", "objective.track",
        "checkpoint.forest"), {after={"arrive"}}),
    -- The ten authored exit defenders. The barrier holds until all of them are confirmed dead.
    step("defense_reached", "forest.defense.reached", {after={"prepare"}}),
    step("defense", "forest.defense", {after={"defense_reached"}}),
    step("portal_seen", "forest.exit.reached", {after={"prepare"}}),
    step("exit_checkpoint", parallel("checkpoint.forest_exit", "objective.barrier"),
        {after={"portal_seen"}}),
    step("cleared", "forest.cleared", {after={"defense"}}),
    step("open", parallel("forest.shield.drop", "objective.traverse"),
        {after={"cleared", "exit_checkpoint"}}),
    step("leaving", "forest.departed", {after={"open"}}),
})

-- The Chase. The security grid is authored objects; the native devices own their own timing.
local chase = graph("chase", "The Chase", {
    step("arrive", "region.chase"),
    step("entered", "chase.entered", {after={"arrive"}}),
    step("prepare", parallel("chase.lasers", "objective.eliminate", "checkpoint.chase",
        "chase.firstroom"), {after={"entered"}}),
    step("line", "dialogue.chase", {after={"prepare"}}),
    step("first_clear", "chase.firstroom.cleared", {after={"prepare"}}),
    step("reinforce", "chase.reinforcements", {after={"first_clear"}}),
    step("reinforce_clear", "chase.reinforcements.cleared", {after={"reinforce"}}),
    step("mount_up", parallel("objective.mount_up", "dialogue.mount_up", "checkpoint.chase_cleared"),
        {after={"reinforce_clear"}}),
    step("sparrow", "chase.sparrow.reached", {after={"mount_up"}}),
    step("conflict", parallel("chase.conflict", "objective.find_leader"), {after={"sparrow"}}),
    -- Both authored routes reach the ledge. The jump cue belongs only to its own route and must
    -- not prevent the easy path from handing off to the next region.
    step("departed", "region.ledge", {after={"conflict"}}),
})

-- The bomb ledge. Its arrival wave is placed the moment the region is held, because the authored
-- entrance ledge precedes the section's first player monitor.
-- The ship and the boss share the region but have independent native receipt chains. The
-- boss joins on its own room's prefight and approach, not on outside kills or ship departure.
local boss = graph("boss", "The bomb ledge and Valus Thuun", {
    step("arrive", "region.ledge"),
    step("prepare", parallel("ledge.arrival", "boss.prefight", "checkpoint.ledge", "objective.find_leader",
        "dialogue.ledge"), {after={"arrive"}}),
    step("final", "ledge.final", {after={"prepare"}}),
    step("final_line", parallel("dialogue.ledge_final", "objective.find_map"), {after={"final"}}),
    step("ship_arrival", "ship.arrive", {after={"final"}}),
    step("ship_delivery", "ship.deliver", {after={"ship_arrival"}}),
    step("ship_departure", "ship.depart", {after={"ship_delivery"}}),
    step("ship_retire", "ship.retire", {after={"ship_departure"}}),
    step("approach", "boss.approached", {after={"prepare"}}),
    step("checkpoint", "checkpoint.boss", {after={"approach"}}),
    step("prefight_clear", "boss.prefight.cleared", {after={"prepare"}}),
    -- The boss squad opens with zero ordinary members; its named combatant binding is what
    -- creates the actor, so the two go out together.
    step("begin", parallel("boss.reveal", "objective.defeat", "dialogue.reveal"),
        {after={"prefight_clear", "approach"}}),
    step("reveal_done", "dialogue.scene_finished", {after={"begin"}}),
    step("room1", parallel("boss.fight1", "dialogue.room1"), {after={"begin"}}),
    step("retreat2", "boss.retreat2", {after={"room1"}}),
    step("retreat2_line", "dialogue.retreat2", {after={"retreat2"}}),
    step("room1_clear", "boss.exit1", {after={"room1"}}),
    step("enter2", "boss.room2.entered", {after={"room1_clear"}}),
    step("room2", parallel("boss.fight2", "objective.evade", "dialogue.room2"), {after={"enter2"}}),
    step("retreat3", "boss.retreat3", {after={"room2"}}),
    step("retreat3_line", parallel("dialogue.room3", command("objective.defeat", {id="objective.defeat.final"})), {after={"retreat3"}}),
    step("room2_clear", "boss.exit2", {after={"room2"}}),
    step("enter3", "boss.room3.entered", {after={"room2_clear"}}),
    step("room3", "boss.fight3", {after={"enter3"}}),
    step("dead", "boss.death", {after={"room3"}}),
    step("finish", parallel("mission.finish", "dialogue.dead"), {after={"dead"}}),
})

return mission{
    id = "strike_pact",
    graphs = {composition, opening, forest, chase, boss},
    roles = {mission="composition", opening="opening", ending="boss"},
    entry = "composition", modules = {"opening"}, observations = {"opening.checked"},
    phases = {"opening", "forest", "chase", "boss"},
    conditions = {forestDefenseReached, openingDeparted, forestExitReached,
        forestDeparted, sparrowReached},
    presentation = presentation{binding_tables={
        route={traversal=array{}, objectives=array{}, dialogue={
            {asset="tunnel.entered", row=4, delay_ms=0},
            {asset="forest.entered", row=5, delay_ms=0},
            {asset="forest.first_complete", row=6, delay_ms=0},
            {asset="chase.jump", row=13, delay_ms=0},
            {asset="ledge.see_tree", row=19, delay_ms=0},
        }},
    }, markers={
        {objective="0xDA162298", target="forest_portal"},
        {objective="0x489890F3", target="forest_portal"},
        {objective="0xEA50F954", target="forest_barrier"},
        {objective="0x3D6350FA", target="chase_fight"},
        {objective="0xE133A090", target="chase_exit"},
        {objective="0x574D4C17", target="chase_exit"},
        {objective="0xF611984A", target="boss_approach"},
        {objective="0x59365CA0", target="boss_room3"},
        {objective="0xD1ECAA7B", target="thuun"},
    }},
}
