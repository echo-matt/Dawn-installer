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
local openingArrived = condition("opening.arrived", any_of("landing.entered", "gate.entered"))
local openingDeparted = condition("opening.departed", any_of("tunnel.entered", "region.forest"))
local forestExitReached = condition("forest.exit.reached",
    any_of("forest.portal.seen", "forest.portal.reached", "region.chase"))
local forestDeparted = condition("forest.departed", any_of("forest.leaving", "region.chase"))
local sparrowReached = condition("chase.sparrow.reached", any_of("chase.sparrow", "chase.jump"))

-- The Lighthouse. Approach the gateway, clear the local defense, take the portal.
local opening = graph("opening", "Lighthouse gateway", {
    step("arrival", parallel("objective.approach", "dialogue.gate", "shield.raise", "opening.stage1")),
    step("landing", "opening.arrived", {after={"arrival"}}),
    -- The opening exchange starts with the objective; only formation waits for landing.
    step("briefing", "opening.stage2", {after={"landing"}}),
    step("gate", "gate.entered", {after={"briefing"}}),
    -- Coordinates lock as the player approaches the gateway.
    step("defense", parallel("dialogue.intro", "opening.stage3"), {after={"gate"}}),
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
    step("prepare", parallel("forest.generate", "forest.shield.raise", "objective.traverse",
        "checkpoint.forest"), {after={"arrive"}}),
    -- The ten authored exit defenders. The barrier holds until all of them are confirmed dead.
    step("defense_reached", "forest.defense.reached", {after={"prepare"}}),
    step("defense", "forest.defense", {after={"defense_reached"}}),
    step("portal_seen", "forest.exit.reached", {after={"prepare"}}),
    step("exit_checkpoint", parallel("checkpoint.forest_exit", "objective.barrier"),
        {after={"portal_seen"}}),
    step("cleared", "forest.cleared", {after={"defense"}}),
    step("open", parallel("forest.shield.drop", command("objective.traverse", {id="forest.resume_objective"})),
        {after={"cleared", "exit_checkpoint"}}),
    step("leaving", "forest.departed", {after={"open"}}),
})

-- The Chase. The security grid is authored objects; the native devices own their own timing.
local chase = graph("chase", "The Chase", {
    step("arrive", "region.chase"),
    step("entered", "chase.entered", {after={"arrive"}}),
    step("prepare", parallel("chase.lasers", "objective.eliminate", "checkpoint.chase",
        "chase.firstroom"), {after={"entered"}}),
    step("first_clear", "chase.firstroom.cleared", {after={"prepare"}}),
    step("reinforce", "chase.reinforcements", {after={"first_clear"}}),
    step("reinforce_clear", "chase.reinforcements.cleared", {after={"reinforce"}}),
    step("mount_up", parallel("objective.mount_up", "dialogue.jump", "checkpoint.chase_cleared"),
        {after={"reinforce_clear"}}),
    step("sparrow", "chase.sparrow.reached", {after={"mount_up"}}),
    step("conflict", parallel("chase.conflict", "objective.find_leader"), {after={"sparrow"}}),
    -- Both authored routes reach the ledge; the Sparrow cue accompanies Mount Up.
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
    step("begin", "boss.reveal", {after={"prefight_clear", "approach"}}),
    -- The reveal command completes at scene end. Watch its live participants in
    -- parallel so the Minotaur line and objective change follow spawn and death.
    step("minotaur_spawn", "boss.minotaur.spawned", {after={"prepare"}}),
    step("minotaur_line", "dialogue.reveal", {after={"minotaur_spawn"}}),
    step("minotaur_death", "boss.minotaur.dead", {after={"minotaur_spawn"}}),
    step("thuun_revealed", parallel("objective.defeat", "dialogue.scene_finished"), {after={"minotaur_death"}}),
    step("room1", "boss.fight1", {after={"begin"}}),
    step("retreat2", "boss.retreat2", {after={"room1"}}),
    step("room1_clear", "boss.exit1", {after={"room1"}}),
    step("enter2", "boss.room2.entered", {after={"room1_clear"}}),
    step("room2", parallel("boss.fight2", "objective.evade"), {after={"enter2"}}),
    step("retreat3", "boss.retreat3", {after={"room2"}}),
    step("retreat3_line", parallel("dialogue.retreat2", command("objective.defeat", {id="objective.defeat.final"})), {after={"retreat3"}}),
    step("room2_clear", "boss.exit2", {after={"room2"}}),
    step("enter3", "boss.room3.entered", {after={"room2_clear"}}),
    step("room3", "boss.fight3", {after={"enter3"}}),
    step("dead", "boss.death", {after={"room3"}}),
    step("access", parallel("objective.access_map", "scan.arm", "dialogue.dead"), {after={"dead"}}),
    step("scan", "scan.complete", {after={"access"}}),
    step("scan_dialogue", "dialogue.scan", {after={"scan"}}),
    step("scan_dialogue_finished", "dialogue.scan.finished", {after={"scan_dialogue"}}),
    step("finish", "mission.finish", {after={"scan_dialogue_finished"}}),
})

return mission{
    id = "mission_pact",
    graphs = {composition, opening, forest, chase, boss},
    roles = {mission="composition", opening="opening", ending="boss"},
    entry = "composition", modules = {"opening"}, observations = {"opening.checked"},
    phases = {"opening", "forest", "chase", "boss"},
    conditions = {forestDefenseReached, openingArrived, openingDeparted, forestExitReached,
        forestDeparted, sparrowReached},
    -- Campaign branch lengths include native delays; strike rows remain unchanged.
    presentation = presentation{dialogue={rows=array{
        {row=0, selector="0xB35F543C", duration_ms=7756, native_delay_ms=0, scene_owned=false},
        {row=1, selector="0x94358F13", duration_ms=9794, native_delay_ms=0, scene_owned=false},
        {row=2, selector="0x00000000", duration_ms=0, native_delay_ms=0, scene_owned=false},
        {row=3, selector="0x00000000", duration_ms=0, native_delay_ms=0, scene_owned=false},
        {row=4, selector="0xC04A5765", duration_ms=9518, native_delay_ms=0, scene_owned=false},
        {row=5, selector="0x6A30D732", duration_ms=4430, native_delay_ms=0, scene_owned=false},
        {row=6, selector="0xC18E182A", duration_ms=0, native_delay_ms=0, scene_owned=false},
        {row=7, selector="0x00000000", duration_ms=0, native_delay_ms=0, scene_owned=false},
        {row=8, selector="0x00000000", duration_ms=0, native_delay_ms=0, scene_owned=false},
        {row=9, selector="0x00000000", duration_ms=0, native_delay_ms=0, scene_owned=false},
        {row=10, selector="0x81D654A2", duration_ms=9262, native_delay_ms=0, scene_owned=false},
        {row=11, selector="0x110D4DBF", duration_ms=0, native_delay_ms=0, scene_owned=false},
        {row=12, selector="0x00000000", duration_ms=0, native_delay_ms=0, scene_owned=false},
        {row=13, selector="0x4B4B0C95", duration_ms=2253, native_delay_ms=0, scene_owned=false},
        {row=14, selector="0x00000000", duration_ms=0, native_delay_ms=0, scene_owned=false},
        {row=15, selector="0x00000000", duration_ms=0, native_delay_ms=0, scene_owned=false},
        {row=16, selector="0x5276621B", duration_ms=5823, native_delay_ms=0, scene_owned=false},
        {row=17, selector="0x274C8AD6", duration_ms=6164, native_delay_ms=0, scene_owned=false},
        {row=18, selector="0x52AE09A9", duration_ms=3786, native_delay_ms=0, scene_owned=false},
        {row=19, selector="0xA40140B4", duration_ms=10497, native_delay_ms=0, scene_owned=false},
        {row=20, selector="0x84D90F6B", duration_ms=2612, native_delay_ms=0, scene_owned=false},
        {row=21, selector="0xCE4E6824", duration_ms=3368, native_delay_ms=0, scene_owned=false},
        {row=22, selector="0xE5B7C166", duration_ms=0, native_delay_ms=0, scene_owned=false},
        {row=23, selector="0x00000000", duration_ms=0, native_delay_ms=0, scene_owned=false},
        {row=24, selector="0xE10541E0", duration_ms=2646, native_delay_ms=0, scene_owned=false},
        {row=25, selector="0x56B56FBC", duration_ms=0, native_delay_ms=0, scene_owned=false},
        {row=26, selector="0xED18D16A", duration_ms=2423, native_delay_ms=0, scene_owned=false},
        {row=27, selector="0xD1DD2887", duration_ms=2341, native_delay_ms=0, scene_owned=false},
        {row=28, selector="0xB61406F2", duration_ms=3588, native_delay_ms=0, scene_owned=false},
        {row=29, selector="0x8C046411", duration_ms=16899, native_delay_ms=0, scene_owned=false},
    }},binding_tables={
        campaign_route={traversal=array{}, objectives=array{}, dialogue={
            {asset="tunnel.entered", row=5, delay_ms=0},
            {asset="forest.started", row=4, delay_ms=0},
            {asset="chase.entered", row=10, delay_ms=0},
            {asset="boss.approached", row=19, delay_ms=0},
        }},
    }, markers={
        {objective="0xF52F2E37", target="map_terminal"},
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
