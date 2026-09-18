-- Deep Storage / adventure_whisk. Reference VV0h2nulU0k, 00:07-11:15.
-- Native identities are recovered from the installed packages. See the implementation
-- record for count/timing policies and fresh-process acceptance still to perform.
local function pop(name) return "pyramidion.sq_"..name..".spawn" end
local function request(name) return "pyramidion.sq_"..name..".request" end
local function dead(name) return "pyramidion.sq_"..name..".cleared" end
local composition=graph("composition","Deep Storage",{step("mission",parallel("mission.module","mission.checked"))})
local opening=graph("opening","Enter the Pyramidion",{
    step("briefing",parallel("objective.0","dialogue.0.queued")),
    step("entry_devices",parallel("entry.d_pyramidion_vault_door.off","entry.d_dustbowl_whisk_conflux.off","descent_exit.sec2_energy_wall_device.off",
        "entry.pf_sync_plate.o_altar.on","entry.o_dustbowl_whisk_block01.on","entry.o_dustbowl_whisk_block02.on")),
    step("arm_plate","entry.plate.arm",{after={"entry_devices"}}),
    step("plate_charged","entry.plate.charged",{after={"arm_plate"}}),
    step("conflux",parallel("entry.o_dustbowl_whisk_conflux.on","entry.d_dustbowl_whisk_conflux.on","objective.1"),{after={"plate_charged"}}),
    step("trick","dialogue.1.queued",{after={"conflux","briefing"}}),
    step("scan_available","entry.scan.enable",{after={"conflux"}}),
    step("scan_started","entry.scan.started",{after={"scan_available"}}),
    step("welcome","dialogue.2.queued",{after={"scan_started","trick"}}),
    step("scan_complete","entry.scan.finished",{after={"scan_started"}}),
    step("door",parallel("entry.d_pyramidion_vault_door.on","objective.2"),{after={"scan_complete"}}),
    -- Publish every descent source at door opening; native streaming may admit them later.
    step("descent_request1",parallel(request("descent_plat01_goblin01"),request("descent_plat01_goblin02"),request("descent_plat02_fanatic01"),request("descent_plat02_fanatic02"),request("descent_plat03_hobgoblin01"),request("descent_plat03_fanatic01"),request("descent_plat03_fanatic02")),{after={"door"}}),
    step("descent_request2",parallel(request("descent_plat04_hobgoblin01"),request("descent_plat04_hobgoblin02"),request("descent_plat04_fanatic01"),request("descent_plat04_fanatic02"),request("descent_plat05_hobgoblin01"),request("descent_plat05_fanatic01"),request("descent_plat05_fanatic02")),{after={"door"}}),
    step("descent_request3",parallel(request("descent_plat06_hobgoblin01"),request("descent_plat06_hobgoblin02"),request("descent_plat06_goblin01")),{after={"door"}}),
    step("shortcut","dialogue.3.queued",{after={"door","welcome"}}),
    step("entered","entry.tv_io_m_whisk_pyramidion_010_vo",{after={"door"}}),
    step("history","dialogue.4.queued",{after={"entered","shortcut"}}),
})
-- Descent geometry follows retained native volume crossings; its enemy requests
-- already belong to the entrance door. Traversal never waits for their deaths.
local descent=graph("descent","Descend into the Pyramidion",{
    step("start",parallel("objective.3","descent_exit.sec2_energy_wall_device.off","pyramidion.tv_descent_plat01")),
    step("approach2","pyramidion.tv_descent_plat02",{after={"start"}}),
    step("approach3","pyramidion.tv_descent_plat03",{after={"approach2"}}),
    step("reveal4",parallel("pyramidion.o_pyramidion_whisk_block04.on","pyramidion.o_pyramidion_whisk_block04_cov.on"),{after={"approach3"}}),
    step("approach4","pyramidion.tv_descent_plat04",{after={"reveal4"}}),
    step("reveal5",parallel("pyramidion.o_pyramidion_whisk_block05.on","pyramidion.o_pyramidion_whisk_block05_cov.on"),{after={"approach4"}}),
    step("approach5","pyramidion.tv_descent_plat05",{after={"reveal5"}}),
    step("reveal6",parallel("pyramidion.o_pyramidion_whisk_block06.on","pyramidion.o_pyramidion_whisk_block06_cov.on"),{after={"approach5"}}),
    step("approach6","pyramidion.tv_descent_plat06",{after={"reveal6"}}),
    step("hallway","pyramidion.tv_io_m_whisk_pyramidion_060_vo"),
    step("data","dialogue.5.queued",{after={"hallway"}}),
})
-- North and middle fallbacks also retain skipped earlier approach observations.
local gateHydra=condition("gate.hydra",any_of("pyramidion.tv_warpgate_fallback027","pyramidion.tv_warpgate_fallback03","pyramidion.tv_io_m_whisk_pyramidion_085_vo"))
local gateMiddle=condition("gate.middle",any_of("pyramidion.tv_warpgate_fallback02","pyramidion.tv_warpgate_fallback025","gate.hydra"))
local gateArena=condition("gate.arena",any_of("pyramidion.tv_warpgate","pyramidion.tv_warpgate_fallback00","pyramidion.tv_warpgate_fallback01","gate.middle"))
local gateApproach=condition("gate.approach",any_of("pyramidion.tv_warpgate_intro","pyramidion.tv_warpgate_intro_fallback01","pyramidion.tv_warpgate_intro_fallback02","gate.arena"))
local warpgate=graph("warpgate","Find an access point",{
    step("approach","gate.approach"),
    step("intro",pop("warpgate_intro_harpy01"),{after={"approach"}}),
    step("arena","gate.arena"),
    step("setup",parallel("objective.4","dialogue.6.queued","pyramidion.d_pyramidion_whisk_warpgate.off","pyramidion.d_pyramidion_whisk_warpgate_barrier.on"),{after={"arena"}}),
    step("wave1",parallel(pop("warpgate_part1_goblin01"),pop("warpgate_part1_hobgoblin01")),{after={"setup"}}),
    step("middle","gate.middle"),
    step("wave2",parallel(pop("warpgate_part2_minotaur01"),pop("warpgate_part2_minotaur02"),pop("warpgate_part2_harpy01"),pop("warpgate_part2_harpy02"),pop("warpgate_part2_harpy03"),pop("warpgate_part2_harpy04")),{after={"setup","middle"}}),
    step("north","gate.hydra"),
    step("wave3",parallel(pop("warpgate_part3_hydra"),pop("warpgate_part3_goblin01")),{after={"setup","north"}}),
    step("hydra_dead",dead("warpgate_part3_hydra"),{after={"wave3"}}),
    step("gate_open",parallel("pyramidion.d_pyramidion_whisk_warpgate.on","pyramidion.d_pyramidion_whisk_warpgate_barrier.off","dialogue.7.queued"),{after={"hydra_dead"}}),
    -- Contact and transport remain native. The receiving corridor is far below
    -- this arena; standing next to the warp gate cannot complete this step.
    step("arrived","gate_route.tv_go_to_first_teleporter_endpoint.occupied",{after={"gate_open"}}),
})
local corridor=graph("corridor","Traverse the warp gate network",{
    step("arrival",parallel("objective.5","dialogue.9.queued")),
    step("lasers",parallel("pyramidion.o_pyramidion_whisk_lasertrap01.on","pyramidion.o_pyramidion_whisk_lasertrap02.on","pyramidion.o_pyramidion_whisk_lasertrap03.on","pyramidion.o_pyramidion_whisk_lasertrap04.on"),{after={"arrival"}}),
    step("hall","pyramidion.tv_corridor",{after={"arrival"}}),
    step("defenders",parallel(pop("corridor_hobgoblin01"),pop("corridor_hobgoblin02"),pop("corridor_harpy01"),pop("corridor_goblin01")),{after={"hall"}}),
    step("arrived","corridor_route.tv_go_to_second_teleporter_endpoint",{after={"lasers"}}),
})
local cyclopsFront=condition("cyclops.front",any_of("pyramidion.tv_cyclops_fallback02","pyramidion.tv_cyclops_barrier01"))
local cyclopsMiddle=condition("cyclops.middle",any_of("pyramidion.tv_cyclops_fallback01","cyclops.front"))
local cyclopsApproach=condition("cyclops.approach",any_of("pyramidion.tv_cyclops","cyclops.middle"))
local cyclops=graph("cyclops","Overcome the Vex",{
    step("approach","cyclops.approach"),
    step("barrier",parallel("objective.6","pyramidion.o_pyramidion_whisk_barrier01.on","pyramidion.d_pyramidion_whisk_barrier01.on"),{after={"approach"}}),
    step("map_sources",parallel("pyramidion.pf_sync_plate_left.o_altar.on","pyramidion.pf_sync_plate_right.o_altar.on",
        "pyramidion.o_pyramidion_whisk_map_room_lens.on","pyramidion.o_pyramidion_whisk_map_room_block01.on"),{after={"approach"}}),
    step("map_lasers",parallel("pyramidion.o_pyramidion_whisk_map_room_laser_center.on","pyramidion.o_pyramidion_whisk_map_room_laser_left.on","pyramidion.o_pyramidion_whisk_map_room_laser_right.on"),{after={"map_sources"}}),
    step("map_initial",parallel("pyramidion.d_pyramidion_whisk_map_room_lens.on","pyramidion.d_pyramidion_whisk_map_room_laser_center.off","pyramidion.d_pyramidion_whisk_map_room_laser_left.on","pyramidion.d_pyramidion_whisk_map_room_laser_right.on","pyramidion.d_pyramidion_whisk_map_room_laser_catch_left.on","pyramidion.d_pyramidion_whisk_map_room_laser_catch_right.on","pyramidion.d_pyramidion_whisk_map_room_conflux.off"),{after={"map_lasers"}}),
    step("map_covers",parallel("pyramidion.d_pyramidion_whisk_map_room_cov01.on","pyramidion.d_pyramidion_whisk_map_room_cov02.on","pyramidion.d_pyramidion_whisk_map_room_cov03.on","pyramidion.d_pyramidion_whisk_map_room_cov04.on"),{after={"map_initial"}}),
    step("network_protection",pop("cyclops_cyclops"),{after={"barrier"}}),
    step("guards1",parallel(pop("cyclops_goblin01"),pop("cyclops_goblin04"),pop("cyclops_goblin05"),pop("cyclops_back_hobgoblin01"),pop("cyclops_back_hobgoblin04"),pop("cyclops_back_hobgoblin05")),{after={"barrier"}}),
    step("clear1",parallel(dead("cyclops_goblin01"),dead("cyclops_goblin04"),dead("cyclops_goblin05"),dead("cyclops_back_hobgoblin01"),dead("cyclops_back_hobgoblin04"),dead("cyclops_back_hobgoblin05")),{after={"guards1"}}),
    step("middle","cyclops.middle"),
    step("guards2",parallel(pop("cyclops_goblin02"),pop("cyclops_goblin03"),pop("cyclops_goblin06"),pop("cyclops_back_hobgoblin02"),pop("cyclops_back_hobgoblin03"),pop("cyclops_back_hobgoblin06")),{after={"barrier","middle"}}),
    step("clear2",parallel(dead("cyclops_goblin02"),dead("cyclops_goblin03"),dead("cyclops_goblin06"),dead("cyclops_back_hobgoblin02"),dead("cyclops_back_hobgoblin03"),dead("cyclops_back_hobgoblin06")),{after={"guards2"}}),
    step("front","cyclops.front"),
    step("guards3",parallel(pop("cyclops_hobgoblin01"),pop("cyclops_hobgoblin02"),pop("cyclops_minotaur01"),pop("cyclops_minotaur02"),pop("cyclops_minotaur03"),pop("cyclops_harpy01")),{after={"barrier","front"}}),
    step("clear3",parallel(dead("cyclops_hobgoblin01"),dead("cyclops_hobgoblin02"),dead("cyclops_minotaur01"),dead("cyclops_minotaur02"),dead("cyclops_minotaur03"),dead("cyclops_harpy01")),{after={"guards3"}}),
    step("boss_dead",dead("cyclops_cyclops"),{after={"network_protection"}}),
    step("barrier_down","pyramidion.d_pyramidion_whisk_barrier01.off",{after={"boss_dead","clear1","clear2","clear3"}}),
    step("pit","final_route.tv_enter_final_space_music",{after={"barrier_down"}}),
    step("darkness",parallel("respawn.restrict","dialogue.10.queued"),{after={"pit"}}),
    step("floor","final.tv_final_room",{after={"darkness"}}),
})
local firstPlate=condition("map.first_plate",any_of("map.left.occupied","map.right.occupied"))
local map=graph("map_room","Activate the conflux",{
    step("setup","objective.7"),
    step("arm",parallel("map.left.arm","map.right.arm"),{after={"setup"}}),
    step("plate","map.first_plate",{after={"arm"}}),
    -- Each plate owns its two waves, independent of either plate charge.
    step("incoming",parallel("dialogue.12.queued",pop("map_room_fanatic_simmer_center")),{after={"plate"}}),
    step("left_enter","map.left.occupied",{after={"arm"}}),
    step("left_wave1",parallel(pop("map_room_fanatic_simmer_left"),pop("map_room_fanatic01_starter"),pop("map_room_fanatic01"),pop("map_room_fanatic03_starter"),pop("map_room_fanatic03")),{after={"left_enter"}}),
    step("left_clear1",parallel(dead("map_room_fanatic_simmer_left"),dead("map_room_fanatic01_starter"),dead("map_room_fanatic01"),dead("map_room_fanatic03_starter"),dead("map_room_fanatic03")),{after={"left_wave1"}}),
    step("left_wave2",parallel(pop("map_room_fanatic05"),pop("map_room_fanatic07"),pop("map_room_hydra01"),pop("map_room_harpy01"),pop("map_room_minotaur01")),{after={"left_clear1"}}),
    step("right_enter","map.right.occupied",{after={"arm"}}),
    step("right_wave1",parallel(pop("map_room_fanatic_simmer_right"),pop("map_room_fanatic02_starter"),pop("map_room_fanatic02"),pop("map_room_fanatic04_starter"),pop("map_room_fanatic04")),{after={"right_enter"}}),
    step("right_clear1",parallel(dead("map_room_fanatic_simmer_right"),dead("map_room_fanatic02_starter"),dead("map_room_fanatic02"),dead("map_room_fanatic04_starter"),dead("map_room_fanatic04")),{after={"right_wave1"}}),
    step("right_wave2",parallel(pop("map_room_fanatic06"),pop("map_room_fanatic08"),pop("map_room_hydra02"),pop("map_room_harpy02"),pop("map_room_minotaur02")),{after={"right_clear1"}}),
    step("left_charge","map.left.charged",{after={"arm"}}),
    step("left_beam",parallel("pyramidion.d_pyramidion_whisk_map_room_laser_left.off","pyramidion.d_pyramidion_whisk_map_room_laser_catch_left.off"),{after={"left_charge"}}),
    step("right_charge","map.right.charged",{after={"arm"}}),
    step("right_beam",parallel("pyramidion.d_pyramidion_whisk_map_room_laser_right.off","pyramidion.d_pyramidion_whisk_map_room_laser_catch_right.off"),{after={"right_charge"}}),
    step("lens_destroyed","map.lens.destroyed",{after={"left_beam","right_beam"}}),
    step("reveal",parallel("pyramidion.d_pyramidion_whisk_map_room_lens.off","pyramidion.d_pyramidion_whisk_map_room_laser_center.off","pyramidion.o_pyramidion_whisk_map_room_conflux.on"),{after={"lens_destroyed"}}),
    step("open_covers",parallel("pyramidion.d_pyramidion_whisk_map_room_cov01.off","pyramidion.d_pyramidion_whisk_map_room_cov02.off","pyramidion.d_pyramidion_whisk_map_room_cov03.off","pyramidion.d_pyramidion_whisk_map_room_cov04.off"),{after={"reveal"}}),
    step("conflux",parallel("pyramidion.d_pyramidion_whisk_map_room_conflux.on","dialogue.11.queued","objective.8"),{after={"open_covers"}}),
    step("scan_available","map.scan.enable",{after={"conflux"}}),
    step("scan_started","map.scan.started",{after={"scan_available"}}),
    step("scan_finished","map.scan.finished",{after={"scan_started"}}),
    -- Retire unfinished optional wave branches after the real final scan.
    step("encounter_complete","map.encounter.finish",{after={"scan_finished"}}),
})
local ending=graph("ending","Recover the coordinates",sequence(
    step("missing","dialogue.13"),
    step("keep_looking","dialogue.finished.13"),
    -- Bind the persistent object while hidden before enabling its native graph.
    step("hologram_source",parallel("pyramidion.d_pyramidion_whisk_map_room_probability_dome_hologram.off","pyramidion.o_pyramidion_whisk_map_room_probability_dome_hologram.on")),
    step("coordinates","dialogue.14"),
    step("hologram","pyramidion.d_pyramidion_whisk_map_room_probability_dome_hologram.on"),
    step("full_exchange","dialogue.finished.14"),
    step("complete",parallel("respawn.allow","mission.finish"))
))
return mission{
    id="deep_storage",graphs={composition,opening,descent,warpgate,corridor,cyclops,map,ending},
    roles={mission="composition"},entry="composition",modules={"mission"},observations={"mission.checked"},
    phases={"opening","descent","warpgate","corridor","cyclops","map_room","ending"},
    conditions={gateHydra,gateMiddle,gateArena,gateApproach,cyclopsFront,cyclopsMiddle,cyclopsApproach,firstPlate},
}
