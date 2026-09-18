-- Hijacked / adventure_rumba. User transcript and route; installed native package bindings.
-- Exact thirds and explicit per-category squad sizes are reconstruction choices.
-- Authentic native deaths, plate charge, relocation, and Ghost scan drive progression.
local function m(name,mode) return "mists."..name.."_squad."..(mode or "request") end
local function s(name,mode) return "surface."..name.."_squad."..(mode or "request") end
local function w(name,mode) return "well.sq_"..name.."."..(mode or "request") end
local composition=graph("composition","Hijacked",{step("mission",parallel("mission.module","mission.checked"))})
-- Authored tunnel-mouth music volume is earlier than the Mists streaming entrance.
local tunnel=condition("mists.tunnel",any_of("tangle.tv_entering_the_lair_music","tangle.tv_endpoint","mists.intro_trigger_volume"))
local cleanup=condition("exterior.cleanup",any_of("tangle.cleanup_point","mists.intro_trigger_volume"))
local patrol=condition("patrol.approach",any_of("surface.mists_fallback01_trigger_volume","surface.mists_fallback02_trigger_volume","mists.tunnel"))
local drop=condition("patrol.drop",any_of("tangle.tv_waypoint","mists.tunnel"))
-- A confirmed Well entrance also completes the earlier exterior approach.
local echoesEntry=condition("echoes.entry",any_of("surface.echoes_intro_fallback01_trigger_volume","well.tv_intro"))
local small=condition("small.approach",any_of("mists.sml_arena_trigger_volume","mists.sml_arena_fallback01_trigger_volume","mists.sml_arena_fallback02_trigger_volume"))
local large=condition("large.approach",any_of("mists.lrg_arena_trigger_volume","mind_route.music_tv_boss_seen"))
local visible=condition("mind.visible",any_of("mists.lrg_arena_trigger_volume.after10s","mind_route.music_tv_boss_seen.after10s"))
local retreat1=condition("mind.retreat1",any_of("mind.health.two_thirds",m("hydra_hydra","cleared")))
local retreat2=condition("mind.retreat2",any_of("mind.health.one_third",m("hydra_hydra","cleared")))
local opening=graph("opening","Track down the Entangled Mind",{
    step("briefing",parallel("objective.0","dialogue.0.queued")),
    step("waiting_harpies",parallel(s("mists_harpy01"),s("mists_harpy02"),s("mists_harpy03"))),
    step("tangle","patrol.approach"),
    step("upper_patrol",parallel(s("mists_hobgoblin01"),s("mists_goblin03")),{after={"tangle"}}),
    step("drop","patrol.drop",{after={"tangle"}}),
    step("patrol",parallel(s("mists_goblin01"),s("mists_goblin02")),{after={"drop"}}),
    step("track","dialogue.4.queued",{after={"tangle"}}),
    step("tunnel","mists.tunnel"),
    step("search_objective",parallel("objective.1","dialogue.2.queued"),{after={"tunnel","briefing"}}),
    step("underground","mists.intro_trigger_volume",{after={"tunnel"}}),
    -- Exterior survivors remain part of this run when the player backtracks.
    -- Native streaming reconstruction preserves the authenticated death ledger.
    step("entrance",parallel(m("intro_harpy01"),m("intro_goblin01")),{after={"underground"}}),
})
local cave=graph("cave","Search the Mists",{
    step("approach","small.approach"),
    -- Room entry requests both groups; kills and streaming readiness cannot block traversal.
    step("harpies",parallel(m("sml_arena_harpy01"),m("sml_arena_harpy02"),m("sml_arena_harpy03")),{after={"approach"}}),
    step("followup",parallel(m("sml_arena_goblin01"),m("sml_arena_goblin02"),m("sml_arena_exit01"),m("sml_arena_exit02")),{after={"approach"}}),
    step("arena","large.approach",{after={"approach"}}),
})
local hunt=graph("hunt","Pursue the Entangled Mind",{
    step("setup",parallel("objective.2","mists.o_mists_rumba_barrier01.on","mists.d_mists_rumba_barrier01.on",m("hydra_hydra"))),
    step("first_position",parallel(m("hydra_hydra","spawn"),"mind.position.0.request"),{after={"setup"}}),
    step("visible","mind.visible"),
    step("identified","dialogue.5.queued",{after={"visible"}}),
    step("front",parallel(m("lrg_arena_front01"),m("lrg_arena_mid01"),m("lrg_arena_harpy01")),{after={"setup"}}),
    step("damage1","mind.retreat1",{after={"first_position"}}),
    step("retreat","mind.position.1",{after={"damage1"}}),
    step("pursuit","dialogue.6.queued",{after={"final_position"}}),
    step("back",parallel(m("lrg_arena_back_left01"),m("lrg_arena_back_right01"),m("lrg_arena_back_sniper_left01"),m("lrg_arena_back_sniper_right01"),m("lrg_arena_exit01")),{after={"retreat"}}),
    step("damage2","mind.retreat2",{after={"retreat"}}),
    step("final_position","mind.position.2",{after={"damage2"}}),
    -- Entry after the second retreat releases the final encounter; prior guard kills are optional.
    step("final_guards",parallel("objective.3",m("hydra_goblin01"),m("hydra_goblin02"),m("hydra_goblin03"),m("hydra_minotaur01")),{after={"pursued"}}),
    step("pursued","mists.hydra_trigger_volume",{after={"final_position"}}),
})
local mind=graph("mind","Defeat the Entangled Mind",{
    -- Only the genuine Mind death releases this barrier; its surviving guards do not hold it.
    step("mind_dead",m("hydra_hydra","cleared")),
    -- Standing point-producer patrols are ready before the return journey.
    step("return_patrols",parallel(s("echoes_harpy01"),s("echoes_minotaur01"),s("echoes_hobgoblin02")),{after={"mind_dead"}}),
    step("barrier_down","mists.d_mists_rumba_barrier01.off",{after={"mind_dead"}}),
    -- The reference's core acquisition exchange follows the death reward. There is no
    -- separate pickup source in the recovered activity; do not invent a pickup receipt.
    step("processor",parallel("objective.4","dialogue.9.queued"),{after={"barrier_down"}}),
    step("exit","exit_route.tv_endpoint",{after={"barrier_down"}}),
    step("returned","return_route.tv_teleport.occupied",{after={"exit"}}),
})
local surface=graph("surface","Find a conflux",{
    -- Keep the destination visible before the Well room is reached; scanning arms later.
    step("conflux_visible",parallel("well.o_echoes_rumba_final_conflux.on","well.d_echoes_rumba_final_conflux.on")),
    step("route","objective.5"),
    step("approach","echoes.entry"),
    step("entry_reinforcements",parallel(s("echoes_intro_harpy"),s("echoes_intro_goblin")),{after={"approach"}}),
    step("echoes","surface.echoes_fallback01_trigger_volume"),
    step("zone_reinforcements",parallel(s("echoes_goblin01"),s("echoes_minotaur02"),s("echoes_minotaur03"),s("echoes_harpy02")),{after={"echoes"}}),
    step("well","well_route.tv_nessus_m_rumba_well_of_echoes_010_vo"),
    step("conflux","dialogue.10.queued",{after={"well"}}),
    step("portal","well.tv_intro",{after={"well"}}),
})
local well=graph("well","Reach the conflux",{
    step("entrance",parallel("objective.6",w("intro_goblin01"),w("intro_fanatic01"),w("intro_hobgoblin01"))),
    -- Retain genuine arrivals if the player crosses before this phase starts.
    step("received","well.tv_mid",{after={"entrance"}}),
    step("mid",parallel(w("mid_goblin01"),w("mid_goblin02"),"well.pf_sync_plate_central.o_altar.on"),{after={"received"}}),
    step("floor","final_route.tv_endpoint",{after={"mid"}}),
    -- These authored guards occupy the floor below the climb route.
    step("floor_guards",parallel(w("final_goblin00"),w("final_goblin01")),{after={"floor"}}),
    step("arm","well.plate.arm",{after={"floor"}}),
    step("charge","well.plate.charged",{after={"arm"}}),
    -- Current occupancy after the preceding platform is ready reveals the next step.
    -- These are authored landing-area volumes, not a native grounded-contact receipt.
    step("platform1","well.o_echoes_rumba_block01.on",{after={"charge"}}),
    step("raise1","well.d_echoes_rumba_block01.on",{after={"platform1"}}),
    step("block1","well.tv_final_block01.occupied",{after={"raise1"}}),
    step("platform2",parallel("well.o_echoes_rumba_block02.on","well.o_echoes_rumba_block02_cov.on"),{after={"block1"}}),
    step("platform2_guards",w("final_plat_goblin01"),{after={"platform2"}}),
    step("block2","well.tv_final_block02.occupied",{after={"platform2"}}),
    step("platform3","well.o_echoes_rumba_block03.on",{after={"block2"}}),
    step("platform3_guards",w("final_plat_goblin02"),{after={"platform3"}}),
    step("block3","well.tv_final_block03.occupied",{after={"platform3"}}),
    step("platform4",parallel("well.o_echoes_rumba_block04.on","well.o_echoes_rumba_block04_cov01.on","well.o_echoes_rumba_block04_cov02.on"),{after={"block3"}}),
    step("raise4","well.d_echoes_rumba_block04.on",{after={"platform4"}}),
    -- The separate sniper support is unused; never add it to the final assembly.
    step("block4","well.tv_final_block04.occupied",{after={"raise4"}}),
    -- Block04 is the final conflux assembly. Block05 is an unused floating slab;
    -- never request it when the player reaches the final landing.

})
local conflux=graph("conflux","Connect the processor to the Vex network",{
    -- Native final_minotaur is a Hydra. Two authored Harpy groups give six escorts.
    step("guards",parallel(w("final_minotaur"),w("final_goblin02"),w("final_harpy01"),w("final_harpy02"))),
    step("clear",parallel(w("final_goblin00","cleared"),w("final_goblin01","cleared"),w("final_minotaur","cleared"),w("final_goblin02","cleared")),{after={"guards"}}),
    step("scan_objective","objective.7",{after={"clear"}}),
    step("scan_available","conflux.scan.enable",{after={"scan_objective"}}),
    step("scan_started","conflux.scan.started",{after={"scan_available"}}),
    step("hope","dialogue.11.queued",{after={"scan_started"}}),
    step("scan_finished","conflux.scan.finished",{after={"scan_started"}}),
    step("complete","well.encounter.finish",{after={"scan_finished","hope"}}),
})
local ending=graph("ending","Find another way",sequence(
    step("image_source",parallel("well.d_echoes_rumba_final_lighthouse.off","well.o_echoes_rumba_final_lighthouse.on")),
    step("processor_failed",parallel("well.d_echoes_rumba_final_lighthouse.on","dialogue.12")),
    step("failed_exchange","dialogue.finished.12"),
    step("past_code","dialogue.13"),
    step("full_exchange","dialogue.finished.13"),
    step("success",parallel("respawn.allow","mission.finish"))
))
return mission{
    id="hijacked",graphs={composition,opening,cave,hunt,mind,surface,well,conflux,ending},
    roles={mission="composition"},entry="composition",modules={"mission"},observations={"mission.checked"},
    phases={"opening","cave","hunt","mind","surface","well","conflux","ending"},
    conditions={tunnel,cleanup,patrol,drop,echoesEntry,small,large,visible,retreat1,retreat2},
}
