-- Beyond Infinity (adventure_vod), reconstructed against the installed package
-- and wzwJ69pzSIE. See docs/BEYOND-INFINITY-IMPLEMENTATION.md for acceptance gaps.
-- Well animations follow native volumes independently of the dialogue queue.
-- Later Scenes retain authored speech and completion gates. Native Daemons own Forest gates.
local composition = graph("composition", "Beyond Infinity", {
    step("mission", parallel("mission.module", "mission.checked")),
})

local entrance = graph("entrance", "Enter the Infinite Forest", sequence(
    step("find_osiris", parallel("objective.0", "dialogue.0", "lighthouse.lighthouse_teleport.on")),
    step("gate", "lighthouse.mercury_m_vod_lighthouse_030_filter"),
    step("vestibule", "well_objectives._directive_initial_volume"),
    step("search", parallel("objective.1", "dialogue.2"))
))

local well = graph("well", "Activate the Well of Echoes", {
    step("prepare", parallel("well.pf_sync_plate.o_altar.on", "well.laser_lense_object.on",
        "well.laser_plate_to_lense_object.on", "well.laser_lense_to_core_object.on")),
    -- Keep the native box shielded and visible before the plate exposes it.
    -- Both laser links stay on while the plate charges its own holographic ring.
    step("mechanism", parallel("well.laser_plate_to_lense_device.on", "well.laser_lense_device.on",
        "well.laser_lense_to_core_device.on", "well.echo_exit_device.on"), {after={"prepare"}}),
    step("light", "well.well_lighting_device.on", {after={"prepare"}}),
    step("climb", "well_objectives._directive_waypoint_volume", {after={"prepare"}}),
    step("search", "dialogue.4", {after={"climb"}}),
    step("plate", "well.plate_occupied", {after={"climb"}}),
    step("lens", "well.lens_destroyed", {after={"plate", "mechanism"}}),
    step("beam", parallel("well.echo_beam1_device.on",
        "well.echo_rings1_device.on", "well.echo_rings2_device.on",
        "well.echo_exit_device.off", "objective.2"), {after={"lens"}}),
    step("reflection", "reflections.scene_echo_first.silent", {after={"beam"}}),
    step("greeting", "dialogue.6.queued", {after={"reflection"}}),
    step("friend", "dialogue.12.queued", {after={"greeting"}}),
    step("reaction", "dialogue.13.queued", {after={"friend"}}),
})

local intro = condition("reflection.intro", any_of("reflections.tv_echo_start", "reflections.tv_echo_start_right"))
local six = condition("reflection.six", any_of("reflections.tv_echo_six", "reflections.tv_echo_six_right"))
local reflections = graph("reflections", "Follow the Reflections of Osiris", {
    -- Animation dependencies contain native crossings and scene bindings only.
    -- Queue acceptance orders the voices without waiting for playback.
    step("intro", "reflection.intro"),
    step("send_word", "reflections.scene_split_2char.silent", {after={"intro"}}),
    step("send_word_voice", "dialogue.14.queued", {after={"send_word"}}),
    step("stairs", "reflections.tv_echo_two", {after={"send_word"}}),
    step("safety", "reflections.scene_echo_intro_two.silent", {after={"stairs"}}),
    step("safety_start", "reflections.scene_echo_intro_two.input.728B4022", {after={"safety"}}),
    step("safety_voice", "dialogue.9.queued", {after={"safety_start", "send_word_voice"}}),
    step("safety_reply", "dialogue.10.queued", {after={"safety_voice"}}),
    -- The high-ledge reflection remains deferred; it cannot block the route.
    step("timelines_volume", "reflections.tv_echo_four", {after={"safety_start"}}),
    step("timelines", "reflections.scene_echo_intro_four_3char.silent", {after={"timelines_volume"}}),
    step("timelines_voice", "dialogue.15.queued", {after={"timelines", "safety_reply"}}),
    step("upper_volume", "reflection.six", {after={"timelines"}}),
    -- Both authored placements animate; the shared line enters the queue once.
    step("upper_pair", parallel("reflections.scene_echo_intro_six.silent",
        "reflections.scene_echo_intro_six_right.silent"), {after={"upper_volume"}}),
    step("upper_start", parallel("reflections.scene_echo_intro_six.input.5C9C66B4",
        "reflections.scene_echo_intro_six_right.input.5C9C66B4"), {after={"upper_pair"}}),
    step("upper_voice", "dialogue.17.queued", {after={"upper_start", "timelines_voice"}}),
    step("corridor", "well_upper.mercury_m_vod_infinite_forest_intro_100_filter", {after={"upper_start"}}),
    step("too_late", "dialogue.19.queued", {after={"corridor", "upper_voice"}}),
    step("too_late_finished", "dialogue.finished.19", {after={"too_late"}}),
    step("precipice", "reveal.tv_precipice_start", {after={"corridor"}}),
    -- Individual run-up scenes2/3 share the paired scene's cast and are alternatives.
    step("runup", parallel("reveal.scene_runup_1", "reveal.scene_runup_2char",
        "reveal.scene_runup_4", "reveal.scene_runup_5", "reveal.scene_runup_6"), {after={"precipice"}}),
    step("reveal", "reveal.scene_if_reveal", {after={"precipice", "too_late_finished"}}),
    step("question_finished", "reveal.scene_if_reveal.dialogue.26.finished", {after={"reveal"}}),
    step("overlook", "forest.tv_begin", {after={"reveal"}}),
    step("behold", "reveal.scene_if_reveal.input.C7ECAA77", {after={"question_finished", "overlook"}}),
    -- The native speech node must run and finish. Root selector completion is
    -- unsuitable here because it retains ambient branches after the dialogue.
    step("behold_finished", "reveal.scene_if_reveal.dialogue.23.finished", {after={"behold"}}),
    -- Retire its retained inputs before a later region reload can replay Behold.
    step("retire_reveal", "reveal.scene_if_reveal.off", {after={"behold_finished"}}),
})

-- Forest A uses the same native solver/Daemon ownership as Omega's adapter,
-- with this mission's north->west and west->east endpoint contracts.
local forest_past = graph("forest_past", "Travel through the Infinite Forest", {
    step("route", parallel("forest.past_route", "objective.3", "forest.door_well_device.on", "dialogue.29")),
    -- This is below the overlook, crossed by jumping into the generated path.
    step("entered", "reveal.tv_if_entered", {after={"route"}}),
    step("daemon_tutorial", "dialogue.30", {after={"entered"}}),
    step("past_gateway", "forest.tv_end_1", {after={"entered"}}),
    step("past_entrance", parallel("past.past_entrance_teleport_object.on", "dialogue.31"), {after={"past_gateway"}}),
    -- The native portal already transports the player. Its actual receiving
    -- area, sampled now, advances the mission even when contact is not sampled.
    step("past_arrived", "past.past_quarantine_volume.occupied", {after={"past_entrance", "daemon_tutorial"}}),
})

local past = graph("past", "Study Mercury's past", {
    step("arrival", "past.past_quarantine_volume"),
    step("study", parallel("objective.4", "past.scene_past_echo", "dialogue.32"), {after={"arrival"}}),
    step("first_history", "dialogue.finished.32", {after={"study"}}),
    step("near_reflection", "past.tv_past_echo", {after={"study"}}),
    step("construction_start", "past.scene_past_echo.input.40C0AC42", {after={"first_history", "near_reflection"}}),
    step("construction_cue", "past.scene_past_echo.event.1FEBC344.emitted", {after={"construction_start"}}),
    step("machines", parallel("past.vex_machine2_object.on", "past.vex_machine3_object.on",
        "past.vex_machine4_object.on", "past.vex_machine5_object.on", "past.vex_machine_rumble_object.on",
        "dialogue.33"), {after={"construction_cue"}}),
    -- Machine1 and the two reflections remain owned by the original Scene.
    step("machine_motion_cue", "past.scene_past_echo.event.C0026857.emitted", {after={"construction_start"}}),
    step("machine_motion", parallel("past.vex_machine2_device.on", "past.vex_machine3_device.on",
        "past.vex_machine4_device.on", "past.vex_machine5_device.on"), {after={"machines", "machine_motion_cue"}}),
    step("panoptes_history", "dialogue.finished.33", {after={"machines"}}),
    step("guardian", "dialogue.34", {after={"panoptes_history"}}),
    step("guardian_finished", "dialogue.finished.34", {after={"guardian"}}),
    step("vista", "past.vignette_start_volume", {after={"study"}}),
    step("history_end", "past.scene_past_echo.input.7AB76D9E", {after={"guardian_finished", "vista", "machine_motion"}}),
    -- The native return portal is released after Osiris's completed line.
    -- The Scene's 51EEA0A6 event did not emit on the verified native route.
    step("portal", parallel("objective.5", "past.vex_teleporter_object.on",
        "past.vex_teleporter_core_object.on", "past.vex_teleporter_device.on"), {after={"guardian_finished"}}),
    -- Native transport lands in the first Forest exit corridor. Require its
    -- current occupancy; the first traversal's old observation cannot return us.
    step("return_arrived", "forest.tv_end_1.occupied", {after={"portal"}}),
    step("forest", "forest.tv_begin_past", {after={"return_arrived"}}),
})

local forest_future = graph("forest_future", "Find Mercury's future", {
    step("route", parallel("forest.future_route", "objective.6")),
    step("prophecies", "dialogue.37", {after={"route"}}),
    step("future_gateway", "forest.tv_end_2", {after={"route"}}),
    -- The warning belongs to the exit corridor, after the Fallen Forest pass.
    step("future_approach", parallel("dialogue.38", "ambush.if_a_teleport_object.on"), {after={"future_gateway"}}),
    -- Loading the Future corridor is not contact with its portal barrier.
    -- Let the native transition/portal move the player; observe actual arrival.
    -- Retain that crossing if the approach dialogue is still finishing.
    step("future_arrived", "future.tv_player_enters_space", {after={"future_approach", "prophecies"}}),
    step("warning_finished", "dialogue.finished.38", {after={"future_approach"}}),
})

local future = graph("future", "Observe the dark future", {
    step("arrival", "future.tv_player_enters_space"),
    step("vision", parallel("objective.7", "future.scene_future_echo", "future.d_vex_eyes.on"), {after={"arrival"}}),
    step("lighthouse", "future.scene_future_echo.input.FA39DB9E", {after={"vision"}}),
    step("lighthouse_finished", "future.scene_future_echo.dialogue.39.finished", {after={"lighthouse"}}),
    -- This native input starts the first reflection's appearance/teleport
    -- vignette. It is not just the next speech: let it run beside Sagira's
    -- opening question as the player crosses the earlier approach volume.
    step("approach_first", "future.tv_player_approaching_first_echo", {after={"vision"}}),
    step("first_reflection", "future.scene_future_echo.input.05BBC301", {after={"approach_first"}}),
    step("darkness_finished", "future.scene_future_echo.dialogue.40.finished", {after={"first_reflection"}}),
    step("near_higher", "future.tv_player_near_echo", {after={"vision"}}),
    -- Native child entity80EC0874 / graph80EC0872 owns rows43,42,44,45.
    -- Its speech, internal relays and reveal delay advance with the Scene.
    step("future_changed", "future.scene_future_echo.input.43E472AF", {after={"lighthouse_finished", "darkness_finished", "near_higher"}}),
    step("panoptes_cue", "future.scene_future_echo.event.3A26E9BE.emitted", {after={"future_changed"}}),
    step("escape_cue", "future.scene_future_echo.event.4A0A18EB.emitted", {after={"panoptes_cue"}}),
    step("escape_speech", "future.scene_future_echo.escape_speech.finished", {after={"escape_cue"}}),
    step("portal", parallel("objective.8", "ambush.lighthouse_teleport_object.on",
        "ambush.lighthouse_teleport2_object.on", "ambush.vex_teleporter_fx_device.on",
        "ambush.vex_teleporter_fx2_device.on"), {after={"escape_speech"}}),
    -- Publish native actor collections and their authored invincibility effects
    -- before any ambush source starts; collection selectors include every actor.
    step("shield_filters", parallel("ambush.vex_invincible_front_filter.on",
        "ambush.vex_invincible_filter.on"), {after={"escape_cue"}}),
    step("shields", parallel("ambush.vex_invincible_front_hopon.on",
        "ambush.vex_invincible_hopon.on"), {after={"shield_filters"}}),
    -- Escape ambush uses the packaged front/back sources and spawn rules.
    -- No enemy-death counter or boss kill is required to enter the exit.
    step("ambush_front", parallel("ambush.future_ambush_front01_squad.on", "ambush.future_ambush_front02_squad.on",
        "ambush.future_ambush_front03_squad.on", "ambush.future_ambush_front04_squad.on",
        "ambush.future_ambush_front05_squad.on", "ambush.future_ambush_front06_squad.on"), {after={"shields"}}),
    step("ambush_back", parallel("ambush.future_ambush_back01_squad.on", "ambush.future_ambush_back02_squad.on",
        "ambush.future_ambush_back03_squad.on"), {after={"shields"}}),
    -- The native escape portal performs the transport. Confirm current presence
    -- in its receiving corridor instead of requesting another host teleport.
    step("escape_arrived", "escape.mercury_m_vod_future_060_filter.occupied", {after={"portal", "ambush_front", "ambush_back"}}),
    -- Preserve real Lighthouse crossings if the player runs ahead of speech.
    step("escaped", parallel("objective.9", "dialogue.47"), {after={"escape_arrived"}}),
})

local escape = graph("escape", "Escape to reality", sequence(
    step("corridor", "escape.mercury_m_vod_future_060_filter"),
    -- Native escape returns directly to present Mercury. The outbound
    -- Infinite Forest intro corridor is in a different coordinate space.
    step("reality", "lighthouse.mercury_m_vod_lighthouse_030_filter"),
    step("ikora", "dialogue.48"),
    step("news", "dialogue.finished.48"),
    step("rendezvous", "objective.10"),
    step("complete", "mission.finish")
))

return mission{
    id="beyond_infinity", graphs={composition, entrance, well, reflections, forest_past, past, forest_future, future, escape},
    roles={mission="composition"}, entry="composition", modules={"mission"}, observations={"mission.checked"},
    phases={"entrance", "well", "reflections", "forest_past", "past", "forest_future", "future", "escape"},
    conditions={intro,six},
}
