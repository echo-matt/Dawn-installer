-- Omega: opening, Forest traversal, Panoptes encounters, and Mercury handoff.
-- Native capability names retain their validated engine identities and receipts.
-- Edit dependencies, parallel commands, dialogue timing, and presentation here.

local function native(capability, id)
    return command(capability, {id=id})
end

-- Mission services and completion milestones.
local mission_graph = graph("mission", "Omega", sequence(
    step("mission services", parallel(
        native("mission/mission services/0", "mission_services.mechanic.0"),
        native("mission/mission services/1", "mission_services.mechanic.1"),
        native("mission/mission services/2", "mission_services.mechanic.2")
    )),
    step("opening and Ikora", native("mission/opening and Ikora/0", "opening_and_ikora.observation.0")),
    step("Forest traversal", native("mission/Forest traversal/0", "forest_traversal.observation.0")),
    step("Lair arrival and encounters", native("mission/Lair arrival and encounters/0", "lair_arrival_and_encounters.observation.0")),
    step("Crown cycle 1", native("mission/Crown cycle 1/0", "crown_cycle_1.observation.0")),
    step("Crown cycle 2", native("mission/Crown cycle 2/0", "crown_cycle_2.observation.0")),
    step("Crown cycle 3", native("mission/Crown cycle 3/0", "crown_cycle_3.observation.0")),
    step("ending cinematic", native("mission/ending cinematic/0", "ending_cinematic.observation.0")),
    step("handoff queued", native("mission/handoff queued/0", "handoff_queued.observation.0"))
), {
    receipts={
        ["opening.complete"]="opening_and_ikora.observation.0",
        ["forest.complete"]="forest_traversal.observation.0",
        ["lair.complete"]="lair_arrival_and_encounters.observation.0",
        ["crown.1.complete"]="crown_cycle_1.observation.0",
        ["crown.2.complete"]="crown_cycle_2.observation.0",
        ["crown.3.complete"]="crown_cycle_3.observation.0",
        ["ending.complete"]="ending_cinematic.observation.0",
        ["handoff.queued"]="handoff_queued.observation.0",
    },
})

-- Ikora and the independent Forest entrance observation.
local opening = graph("opening", "Omega opening", {
    step("roster admitted", parallel(
        native("opening/roster admitted/0", "roster_admitted.observation.0"),
        native("opening/roster admitted/1", "roster_admitted.device.1")
    )),
    step("Ikora approach", native("opening/Ikora approach/0", "ikora_approach.observation.0"), {after={"roster admitted"}}),
    step("Ikora scene ready", native("opening/Ikora scene ready/0", "ikora_scene_ready.scene.0"), {after={"Ikora approach"}}),
    step("Ikora portal output", native("opening/Ikora portal output/0", "ikora_portal_output.observation.0"), {after={"Ikora scene ready"}}),
    step("lattice release requested", native("opening/lattice release requested/0", "lattice_release_requested.device.0"), {after={"Ikora portal output"}}),
    step("Forest entrance observed", native("opening/Forest entrance observed/0", "forest_entrance_observed.observation.0"), {after={"roster admitted"}}),
}, {
    receipts={
        ["roster.admitted"]="roster_admitted.observation.0",
        ["ikora.approached"]="ikora_approach.observation.0",
        ["ikora.scene.ready"]="ikora_scene_ready.scene.0",
        ["ikora.portal.released"]="ikora_portal_output.observation.0",
        ["forest.entered"]="forest_entrance_observed.observation.0",
    },
})

-- Independent landmarks publish their traversal, objective, and dialogue cues.
local forest = graph("forest", "Omega Forest to Lair", {
    step("tunnel reached or passed", native("forest/tunnel reached or passed/0", "tunnel_reached_or_passed.observation.0")),
    step("tunnel presentation", parallel(
        native("forest/tunnel presentation/0", "tunnel_presentation.traversal.0"),
        native("forest/tunnel presentation/1", "tunnel_presentation.objective.1"),
        native("forest/tunnel presentation/2", "tunnel_presentation.dialogue.2")
    ), {after={"tunnel reached or passed"}}),
    step("vista reached or passed", native("forest/vista reached or passed/0", "vista_reached_or_passed.observation.0")),
    step("vista presentation", parallel(
        native("forest/vista presentation/0", "vista_presentation.traversal.0"),
        native("forest/vista presentation/1", "vista_presentation.objective.1"),
        native("forest/vista presentation/2", "vista_presentation.dialogue.2")
    ), {after={"vista reached or passed"}}),
    step("Forest exit reached or passed", native("forest/Forest exit reached or passed/0", "forest_exit_reached_or_passed.observation.0")),
    step("Forest exit presentation", parallel(
        native("forest/Forest exit presentation/0", "forest_exit_presentation.traversal.0"),
        native("forest/Forest exit presentation/1", "forest_exit_presentation.objective.1"),
        native("forest/Forest exit presentation/2", "forest_exit_presentation.dialogue.2")
    ), {after={"Forest exit reached or passed"}}),
    step("Lair reached or passed", native("forest/Lair reached or passed/0", "lair_reached_or_passed.observation.0")),
    step("Lair handoff", parallel(
        native("forest/Lair handoff/0", "lair_handoff.traversal.0"),
        native("forest/Lair handoff/1", "lair_handoff.objective.1")
    ), {after={"Lair reached or passed"}}),
}, {
    receipts={
        ["tunnel.reached"]="tunnel_reached_or_passed.observation.0",
        ["vista.reached"]="vista_reached_or_passed.observation.0",
        ["forest.exit.reached"]="forest_exit_reached_or_passed.observation.0",
        ["lair.reached"]="lair_reached_or_passed.observation.0",
    },
})

-- Initial Panoptes reveal and native camera ownership.
local reveal = graph("reveal", "Panoptes reveal", sequence(
    step("camera and dialogue eligibility", native("reveal/camera and dialogue eligibility/0", "camera_and_dialogue_eligibility.observation.0")),
    step("owned boss flight", parallel(
        native("reveal/owned boss flight/0", "owned_boss_flight.mechanic.0"),
        native("reveal/owned boss flight/1", "owned_boss_flight.observation.1")
    )),
    step("native camera activation", parallel(
        native("reveal/native camera activation/0", "native_camera_activation.cinematic.0"),
        native("reveal/native camera activation/1", "native_camera_activation.observation.1")
    )),
    step("native camera completion", parallel(
        native("reveal/native camera completion/0", "native_camera_completion.cinematic.0"),
        native("reveal/native camera completion/1", "native_camera_completion.observation.1")
    )),
    step("retire reveal", native("reveal/retire reveal/0", "retire_reveal.cinematic.0"))
), {
    receipts={
        ["camera.eligible"]="camera_and_dialogue_eligibility.observation.0",
        ["boss.flight.ready"]="owned_boss_flight.observation.1",
        ["camera.active"]="native_camera_activation.observation.1",
        ["camera.complete"]="native_camera_completion.observation.1",
    },
})

-- Retry the reveal without replaying dialogue eligibility.
local reveal_retry = graph("reveal_retry", "Panoptes reveal retry", sequence(
    step("owned boss flight", parallel(
        native("reveal_retry/owned boss flight/0", "owned_boss_flight.mechanic.0"),
        native("reveal_retry/owned boss flight/1", "owned_boss_flight.observation.1")
    )),
    step("native camera activation", parallel(
        native("reveal_retry/native camera activation/0", "native_camera_activation.cinematic.0"),
        native("reveal_retry/native camera activation/1", "native_camera_activation.observation.1")
    )),
    step("native camera completion", parallel(
        native("reveal_retry/native camera completion/0", "native_camera_completion.cinematic.0"),
        native("reveal_retry/native camera completion/1", "native_camera_completion.observation.1")
    )),
    step("retire reveal", native("reveal_retry/retire reveal/0", "retire_reveal.cinematic.0"))
), {
    receipts={
        ["boss.flight.ready"]="owned_boss_flight.observation.1",
        ["camera.active"]="native_camera_activation.observation.1",
        ["camera.complete"]="native_camera_completion.observation.1",
    },
})

-- Lair encounter and transfer to the first island.
local lair = graph("lair", "kLair", sequence(
    step("native intro summon", native("lair/native intro summon/0", "native_intro_summon.observation.0")),
    step("left summon", parallel(
        native("lair/left summon/0", "left_summon.mechanic.0"),
        native("lair/left summon/1", "left_summon.observation.1")
    )),
    step("right summon", parallel(
        native("lair/right summon/0", "right_summon.mechanic.0"),
        native("lair/right summon/1", "right_summon.observation.1")
    )),
    step("required cohort deaths", native("lair/required cohort deaths/0", "required_cohort_deaths.observation.0")),
    step("fold and path completion", parallel(
        native("lair/fold and path completion/0", "fold_and_path_completion.mechanic.0"),
        native("lair/fold and path completion/1", "fold_and_path_completion.observation.1")
    )),
    step("player arrival", native("lair/player arrival/0", "player_arrival.observation.0")),
    step("enter island A", native("lair/enter island A/0", "enter_island_a.mechanic.0"))
), {
    receipts={
        ["native_intro_summon.observation.0"]="native_intro_summon.observation.0",
        ["left_summon.observation.1"]="left_summon.observation.1",
        ["right_summon.observation.1"]="right_summon.observation.1",
        ["required_cohort_deaths.observation.0"]="required_cohort_deaths.observation.0",
        ["fold_and_path_completion.observation.1"]="fold_and_path_completion.observation.1",
        ["player_arrival.observation.0"]="player_arrival.observation.0",
    },
})

-- First island encounter.
local island_a = graph("island_a", "kIslandA", sequence(
    step("left summon", parallel(
        native("island_a/left summon/0", "left_summon.mechanic.0"),
        native("island_a/left summon/1", "left_summon.observation.1")
    )),
    step("required cohort deaths", native("island_a/required cohort deaths/0", "required_cohort_deaths.observation.0")),
    step("fold and path completion", parallel(
        native("island_a/fold and path completion/0", "fold_and_path_completion.mechanic.0"),
        native("island_a/fold and path completion/1", "fold_and_path_completion.observation.1")
    )),
    step("player arrival", native("island_a/player arrival/0", "player_arrival.observation.0")),
    step("enter island B", native("island_a/enter island B/0", "enter_island_b.mechanic.0"))
), {
    receipts={
        ["left_summon.observation.1"]="left_summon.observation.1",
        ["required_cohort_deaths.observation.0"]="required_cohort_deaths.observation.0",
        ["fold_and_path_completion.observation.1"]="fold_and_path_completion.observation.1",
        ["player_arrival.observation.0"]="player_arrival.observation.0",
    },
})

-- Second island encounter.
local island_b = graph("island_b", "kIslandB", sequence(
    step("right summon", parallel(
        native("island_b/right summon/0", "right_summon.mechanic.0"),
        native("island_b/right summon/1", "right_summon.observation.1")
    )),
    step("required cohort deaths", native("island_b/required cohort deaths/0", "required_cohort_deaths.observation.0")),
    step("fold and path completion", parallel(
        native("island_b/fold and path completion/0", "fold_and_path_completion.mechanic.0"),
        native("island_b/fold and path completion/1", "fold_and_path_completion.observation.1")
    )),
    step("player arrival", native("island_b/player arrival/0", "player_arrival.observation.0")),
    step("enter island C", native("island_b/enter island C/0", "enter_island_c.mechanic.0"))
), {
    receipts={
        ["right_summon.observation.1"]="right_summon.observation.1",
        ["required_cohort_deaths.observation.0"]="required_cohort_deaths.observation.0",
        ["fold_and_path_completion.observation.1"]="fold_and_path_completion.observation.1",
        ["player_arrival.observation.0"]="player_arrival.observation.0",
    },
})

-- Third island encounter and Crown entry.
local island_c = graph("island_c", "kIslandC", sequence(
    step("left summon", parallel(
        native("island_c/left summon/0", "left_summon.mechanic.0"),
        native("island_c/left summon/1", "left_summon.observation.1")
    )),
    step("required cohort deaths", native("island_c/required cohort deaths/0", "required_cohort_deaths.observation.0")),
    step("fold and path completion", parallel(
        native("island_c/fold and path completion/0", "fold_and_path_completion.mechanic.0"),
        native("island_c/fold and path completion/1", "fold_and_path_completion.observation.1")
    )),
    step("Crown arrival and cannon preparation", native("island_c/Crown arrival and cannon preparation/0", "crown_arrival_and_cannon_preparation.observation.0")),
    step("enter Crown", native("island_c/enter Crown/0", "enter_crown.mechanic.0"))
), {
    receipts={
        ["left_summon.observation.1"]="left_summon.observation.1",
        ["required_cohort_deaths.observation.0"]="required_cohort_deaths.observation.0",
        ["fold_and_path_completion.observation.1"]="fold_and_path_completion.observation.1",
        ["crown_arrival_and_cannon_preparation.observation.0"]="crown_arrival_and_cannon_preparation.observation.0",
    },
})

-- First Crown cycle: waves, rescue, charge, exposure, and recovery.
local crown_1 = graph("crown_1", "kCrown1", sequence(
    step("both summon", parallel(
        native("crown_1/both summon/0", "both_summon.mechanic.0"),
        native("crown_1/both summon/1", "both_summon.observation.1")
    )),
    step("first wave deaths", native("crown_1/first wave deaths/0", "first_wave_deaths.observation.0")),
    step("select second wave", native("crown_1/select second wave/0", "select_second_wave.mechanic.0")),
    step("left summon", parallel(
        native("crown_1/left summon/0", "left_summon.mechanic.0"),
        native("crown_1/left summon/1", "left_summon.observation.1")
    )),
    step("second wave deaths", native("crown_1/second wave deaths/0", "second_wave_deaths.observation.0")),
    step("select third wave", native("crown_1/select third wave/0", "select_third_wave.mechanic.0")),
    step("right summon", parallel(
        native("crown_1/right summon/0", "right_summon.mechanic.0"),
        native("crown_1/right summon/1", "right_summon.observation.1")
    )),
    step("third wave deaths", native("crown_1/third wave deaths/0", "third_wave_deaths.observation.0")),
    step("deletion animation", parallel(
        native("crown_1/deletion animation/0", "deletion_animation.mechanic.0"),
        native("crown_1/deletion animation/1", "deletion_animation.observation.1")
    )),
    step("Osiris rescue Scene", parallel(
        native("crown_1/Osiris rescue Scene/0", "osiris_rescue_scene.mechanic.0"),
        native("crown_1/Osiris rescue Scene/1", "osiris_rescue_scene.observation.1")
    )),
    step("charge route and dunk", parallel(
        native("crown_1/charge route and dunk/0", "charge_route_and_dunk.mechanic.0"),
        native("crown_1/charge route and dunk/1", "charge_route_and_dunk.observation.1")
    )),
    step("native eye exposure", parallel(
        native("crown_1/native eye exposure/0", "native_eye_exposure.mechanic.0"),
        native("crown_1/native eye exposure/1", "native_eye_exposure.observation.1")
    )),
    step("native eye threshold", native("crown_1/native eye threshold/0", "native_eye_threshold.observation.0")),
    step("recovery join", parallel(
        native("crown_1/recovery join/0", "recovery_join.mechanic.0"),
        native("crown_1/recovery join/1", "recovery_join.observation.1"),
        native("crown_1/recovery join/2", "recovery_join.observation.2"),
        native("crown_1/recovery join/3", "recovery_join.observation.3")
    )),
    step("enter second cycle", native("crown_1/enter second cycle/0", "enter_second_cycle.mechanic.0"))
), {
    receipts={
        ["both_summon.observation.1"]="both_summon.observation.1",
        ["first_wave_deaths.observation.0"]="first_wave_deaths.observation.0",
        ["left_summon.observation.1"]="left_summon.observation.1",
        ["second_wave_deaths.observation.0"]="second_wave_deaths.observation.0",
        ["right_summon.observation.1"]="right_summon.observation.1",
        ["third_wave_deaths.observation.0"]="third_wave_deaths.observation.0",
        ["deletion_animation.observation.1"]="deletion_animation.observation.1",
        ["osiris_rescue_scene.observation.1"]="osiris_rescue_scene.observation.1",
        ["charge_route_and_dunk.observation.1"]="charge_route_and_dunk.observation.1",
        ["native_eye_exposure.observation.1"]="native_eye_exposure.observation.1",
        ["native_eye_threshold.observation.0"]="native_eye_threshold.observation.0",
        ["recovery_join.observation.1"]="recovery_join.observation.1",
        ["recovery_join.observation.2"]="recovery_join.observation.2",
        ["recovery_join.observation.3"]="recovery_join.observation.3",
    },
})

-- Second Crown cycle and final relocation.
local crown_2 = graph("crown_2", "kCrown2", sequence(
    step("both summon", parallel(
        native("crown_2/both summon/0", "both_summon.mechanic.0"),
        native("crown_2/both summon/1", "both_summon.observation.1")
    )),
    step("first wave deaths", native("crown_2/first wave deaths/0", "first_wave_deaths.observation.0")),
    step("select second wave", native("crown_2/select second wave/0", "select_second_wave.mechanic.0")),
    step("left summon", parallel(
        native("crown_2/left summon/0", "left_summon.mechanic.0"),
        native("crown_2/left summon/1", "left_summon.observation.1")
    )),
    step("second wave deaths", native("crown_2/second wave deaths/0", "second_wave_deaths.observation.0")),
    step("select third wave", native("crown_2/select third wave/0", "select_third_wave.mechanic.0")),
    step("right summon", parallel(
        native("crown_2/right summon/0", "right_summon.mechanic.0"),
        native("crown_2/right summon/1", "right_summon.observation.1")
    )),
    step("third wave deaths", native("crown_2/third wave deaths/0", "third_wave_deaths.observation.0")),
    step("deletion animation", parallel(
        native("crown_2/deletion animation/0", "deletion_animation.mechanic.0"),
        native("crown_2/deletion animation/1", "deletion_animation.observation.1")
    )),
    step("Osiris rescue Scene", parallel(
        native("crown_2/Osiris rescue Scene/0", "osiris_rescue_scene.mechanic.0"),
        native("crown_2/Osiris rescue Scene/1", "osiris_rescue_scene.observation.1")
    )),
    step("charge route and dunk", parallel(
        native("crown_2/charge route and dunk/0", "charge_route_and_dunk.mechanic.0"),
        native("crown_2/charge route and dunk/1", "charge_route_and_dunk.observation.1")
    )),
    step("native eye exposure", parallel(
        native("crown_2/native eye exposure/0", "native_eye_exposure.mechanic.0"),
        native("crown_2/native eye exposure/1", "native_eye_exposure.observation.1")
    )),
    step("native eye threshold", native("crown_2/native eye threshold/0", "native_eye_threshold.observation.0")),
    step("recovery join", parallel(
        native("crown_2/recovery join/0", "recovery_join.mechanic.0"),
        native("crown_2/recovery join/1", "recovery_join.observation.1"),
        native("crown_2/recovery join/2", "recovery_join.observation.2"),
        native("crown_2/recovery join/3", "recovery_join.observation.3")
    )),
    step("escape cohort", native("crown_2/escape cohort/0", "escape_cohort.mechanic.0")),
    step("escape summon completion", parallel(
        native("crown_2/escape summon completion/0", "escape_summon_completion.mechanic.0"),
        native("crown_2/escape summon completion/1", "escape_summon_completion.observation.1")
    )),
    step("final relocation fold and path", parallel(
        native("crown_2/final relocation fold and path/0", "final_relocation_fold_and_path.mechanic.0"),
        native("crown_2/final relocation fold and path/1", "final_relocation_fold_and_path.observation.1")
    )),
    step("final cannon Scene and player arrival", parallel(
        native("crown_2/final cannon Scene and player arrival/0", "final_cannon_scene_and_player_arrival.mechanic.0"),
        native("crown_2/final cannon Scene and player arrival/1", "final_cannon_scene_and_player_arrival.observation.1")
    )),
    step("enter third cycle", native("crown_2/enter third cycle/0", "enter_third_cycle.mechanic.0"))
), {
    receipts={
        ["both_summon.observation.1"]="both_summon.observation.1",
        ["first_wave_deaths.observation.0"]="first_wave_deaths.observation.0",
        ["left_summon.observation.1"]="left_summon.observation.1",
        ["second_wave_deaths.observation.0"]="second_wave_deaths.observation.0",
        ["right_summon.observation.1"]="right_summon.observation.1",
        ["third_wave_deaths.observation.0"]="third_wave_deaths.observation.0",
        ["deletion_animation.observation.1"]="deletion_animation.observation.1",
        ["osiris_rescue_scene.observation.1"]="osiris_rescue_scene.observation.1",
        ["charge_route_and_dunk.observation.1"]="charge_route_and_dunk.observation.1",
        ["native_eye_exposure.observation.1"]="native_eye_exposure.observation.1",
        ["native_eye_threshold.observation.0"]="native_eye_threshold.observation.0",
        ["recovery_join.observation.1"]="recovery_join.observation.1",
        ["recovery_join.observation.2"]="recovery_join.observation.2",
        ["recovery_join.observation.3"]="recovery_join.observation.3",
        ["escape_summon_completion.observation.1"]="escape_summon_completion.observation.1",
        ["final_relocation_fold_and_path.observation.1"]="final_relocation_fold_and_path.observation.1",
        ["final_cannon_scene_and_player_arrival.observation.1"]="final_cannon_scene_and_player_arrival.observation.1",
    },
})

-- Final Crown cycle, native death, and ending handoff.
local crown_3 = graph("crown_3", "kCrown3", sequence(
    step("both summon", parallel(
        native("crown_3/both summon/0", "both_summon.mechanic.0"),
        native("crown_3/both summon/1", "both_summon.observation.1")
    )),
    step("first wave deaths", native("crown_3/first wave deaths/0", "first_wave_deaths.observation.0")),
    step("select second wave", native("crown_3/select second wave/0", "select_second_wave.mechanic.0")),
    step("left summon", parallel(
        native("crown_3/left summon/0", "left_summon.mechanic.0"),
        native("crown_3/left summon/1", "left_summon.observation.1")
    )),
    step("second wave deaths", native("crown_3/second wave deaths/0", "second_wave_deaths.observation.0")),
    step("select third wave", native("crown_3/select third wave/0", "select_third_wave.mechanic.0")),
    step("right summon", parallel(
        native("crown_3/right summon/0", "right_summon.mechanic.0"),
        native("crown_3/right summon/1", "right_summon.observation.1")
    )),
    step("third wave deaths", native("crown_3/third wave deaths/0", "third_wave_deaths.observation.0")),
    step("deletion animation", parallel(
        native("crown_3/deletion animation/0", "deletion_animation.mechanic.0"),
        native("crown_3/deletion animation/1", "deletion_animation.observation.1")
    )),
    step("Osiris rescue Scene", parallel(
        native("crown_3/Osiris rescue Scene/0", "osiris_rescue_scene.mechanic.0"),
        native("crown_3/Osiris rescue Scene/1", "osiris_rescue_scene.observation.1")
    )),
    step("charge route and dunk", parallel(
        native("crown_3/charge route and dunk/0", "charge_route_and_dunk.mechanic.0"),
        native("crown_3/charge route and dunk/1", "charge_route_and_dunk.observation.1")
    )),
    step("native eye exposure", parallel(
        native("crown_3/native eye exposure/0", "native_eye_exposure.mechanic.0"),
        native("crown_3/native eye exposure/1", "native_eye_exposure.observation.1")
    )),
    step("native eye threshold", native("crown_3/native eye threshold/0", "native_eye_threshold.observation.0")),
    step("native death and animation join", parallel(
        native("crown_3/native death and animation join/0", "native_death_and_animation_join.mechanic.0"),
        native("crown_3/native death and animation join/1", "native_death_and_animation_join.observation.1"),
        native("crown_3/native death and animation join/2", "native_death_and_animation_join.observation.2")
    )),
    step("ending handoff", native("crown_3/ending handoff/0", "ending_handoff.mechanic.0"))
), {
    receipts={
        ["both_summon.observation.1"]="both_summon.observation.1",
        ["first_wave_deaths.observation.0"]="first_wave_deaths.observation.0",
        ["left_summon.observation.1"]="left_summon.observation.1",
        ["second_wave_deaths.observation.0"]="second_wave_deaths.observation.0",
        ["right_summon.observation.1"]="right_summon.observation.1",
        ["third_wave_deaths.observation.0"]="third_wave_deaths.observation.0",
        ["deletion_animation.observation.1"]="deletion_animation.observation.1",
        ["osiris_rescue_scene.observation.1"]="osiris_rescue_scene.observation.1",
        ["charge_route_and_dunk.observation.1"]="charge_route_and_dunk.observation.1",
        ["native_eye_exposure.observation.1"]="native_eye_exposure.observation.1",
        ["native_eye_threshold.observation.0"]="native_eye_threshold.observation.0",
        ["native_death_and_animation_join.observation.1"]="native_death_and_animation_join.observation.1",
        ["native_death_and_animation_join.observation.2"]="native_death_and_animation_join.observation.2",
    },
})

-- Retire the Lair, play the ending, and queue Mercury.
local ending = graph("ending", "Omega ending and handoff", sequence(
    step("final dialogue", native("ending/final dialogue/0", "final_dialogue.observation.0")),
    step("native Lair retirement", parallel(
        native("ending/native Lair retirement/0", "native_lair_retirement.mechanic.0"),
        native("ending/native Lair retirement/1", "native_lair_retirement.observation.1")
    )),
    step("bookend arrival and native camera eligibility", parallel(
        native("ending/bookend arrival and native camera eligibility/0", "bookend_arrival_and_native_camera_eligibility.traversal.0"),
        native("ending/bookend arrival and native camera eligibility/1", "bookend_arrival_and_native_camera_eligibility.observation.1")
    )),
    step("native cinematic activation", parallel(
        native("ending/native cinematic activation/0", "native_cinematic_activation.cinematic.0"),
        native("ending/native cinematic activation/1", "native_cinematic_activation.observation.1")
    )),
    step("native cinematic completion", parallel(
        native("ending/native cinematic completion/0", "native_cinematic_completion.cinematic.0"),
        native("ending/native cinematic completion/1", "native_cinematic_completion.observation.1")
    )),
    step("retire cinematic command", native("ending/retire cinematic command/0", "retire_cinematic_command.cinematic.0")),
    step("native Mercury launch queued", parallel(
        native("ending/native Mercury launch queued/0", "native_mercury_launch_queued.traversal.0"),
        native("ending/native Mercury launch queued/1", "native_mercury_launch_queued.observation.1")
    ))
), {
    receipts={
        ["dialogue.complete"]="final_dialogue.observation.0",
        ["lair.retired"]="native_lair_retirement.observation.1",
        ["camera.eligible"]="bookend_arrival_and_native_camera_eligibility.observation.1",
        ["camera.active"]="native_cinematic_activation.observation.1",
        ["camera.complete"]="native_cinematic_completion.observation.1",
        ["handoff.queued"]="native_mercury_launch_queued.observation.1",
    },
})

-- Retry camera ownership without replaying final dialogue.
local ending_retry = graph("ending_retry", "Omega ending camera retry", sequence(
    step("native camera eligibility", native("ending_retry/native camera eligibility/0", "native_camera_eligibility.observation.0")),
    step("native cinematic activation", parallel(
        native("ending_retry/native cinematic activation/0", "native_cinematic_activation.cinematic.0"),
        native("ending_retry/native cinematic activation/1", "native_cinematic_activation.observation.1")
    )),
    step("native cinematic completion", parallel(
        native("ending_retry/native cinematic completion/0", "native_cinematic_completion.cinematic.0"),
        native("ending_retry/native cinematic completion/1", "native_cinematic_completion.observation.1")
    )),
    step("retire cinematic command", native("ending_retry/retire cinematic command/0", "retire_cinematic_command.cinematic.0")),
    step("native Mercury launch queued", parallel(
        native("ending_retry/native Mercury launch queued/0", "native_mercury_launch_queued.traversal.0"),
        native("ending_retry/native Mercury launch queued/1", "native_mercury_launch_queued.observation.1")
    ))
), {
    receipts={
        ["camera.eligible"]="native_camera_eligibility.observation.0",
        ["camera.active"]="native_cinematic_activation.observation.1",
        ["camera.complete"]="native_cinematic_completion.observation.1",
        ["handoff.queued"]="native_mercury_launch_queued.observation.1",
    },
})

return mission{
    id="omega",
    graphs={mission_graph, opening, forest, reveal, reveal_retry, lair, island_a, island_b, island_c, crown_1, crown_2, crown_3, ending, ending_retry},
    roles={
        mission="mission",
        opening="opening",
        forest="forest",
        reveal="reveal",
        reveal_retry="reveal_retry",
        lair="lair",
        island_a="island_a",
        island_b="island_b",
        island_c="island_c",
        crown_1="crown_1",
        crown_2="crown_2",
        crown_3="crown_3",
        ending="ending",
        ending_retry="ending_retry",
    },
    entry="mission",
    modules={"presentation", "panoptes", "ending"},
    observations={"opening.complete", "forest.complete", "lair.complete", "crown.1.complete", "crown.2.complete", "crown.3.complete", "ending.complete", "handoff.queued"},
    presentation=presentation{
        dialogue={
            bank="0x80F1FD07",
            dispatch_timeout_ms=15000,
            spacing_ms=250,
            rows={
                {row=0, selector="0xAD60F465", duration_ms=5445, native_delay_ms=1000, scene_owned=false},
                {row=1, selector="0x57477432", duration_ms=6083, native_delay_ms=0, scene_owned=true},
                {row=2, selector="0x730F03C7", duration_ms=1985, native_delay_ms=0, scene_owned=false},
                {row=3, selector="0x47AF17F4", duration_ms=5051, native_delay_ms=0, scene_owned=true},
                {row=4, selector="0xB4C3F0B9", duration_ms=3218, native_delay_ms=0, scene_owned=true},
                {row=5, selector="0x0ED8C762", duration_ms=0, native_delay_ms=0, scene_owned=false},
                {row=6, selector="0xAE2495AC", duration_ms=8093, native_delay_ms=0, scene_owned=false},
                {row=7, selector="0x0E9C80BE", duration_ms=10990, native_delay_ms=0, scene_owned=false},
                {row=8, selector="0x08AE5FB8", duration_ms=0, native_delay_ms=0, scene_owned=false},
                {row=9, selector="0xE878194A", duration_ms=6461, native_delay_ms=0, scene_owned=false},
                {row=10, selector="0x7BA4F101", duration_ms=0, native_delay_ms=0, scene_owned=false},
                {row=11, selector="0xB2CF9D6E", duration_ms=0, native_delay_ms=0, scene_owned=false},
                {row=12, selector="0xAB0676A8", duration_ms=2019, native_delay_ms=0, scene_owned=false},
                {row=13, selector="0xA558F78F", duration_ms=6222, native_delay_ms=0, scene_owned=false},
                {row=14, selector="0x94E09524", duration_ms=5761, native_delay_ms=0, scene_owned=false},
                {row=15, selector="0x0294D229", duration_ms=4432, native_delay_ms=0, scene_owned=false},
                {row=16, selector="0xB8CE809F", duration_ms=6354, native_delay_ms=0, scene_owned=false},
                {row=17, selector="0x9DA4C20A", duration_ms=0, native_delay_ms=0, scene_owned=false},
                {row=18, selector="0xCB7F171D", duration_ms=2301, native_delay_ms=0, scene_owned=false},
                {row=19, selector="0xC0570578", duration_ms=0, native_delay_ms=0, scene_owned=false},
                {row=20, selector="0xD16ECB03", duration_ms=0, native_delay_ms=0, scene_owned=false},
                {row=21, selector="0xC645267E", duration_ms=3538, native_delay_ms=0, scene_owned=false},
                {row=22, selector="0x202F7829", duration_ms=3320, native_delay_ms=0, scene_owned=false},
                {row=23, selector="0xD4CADE1D", duration_ms=2486, native_delay_ms=0, scene_owned=true},
                {row=24, selector="0x1C653216", duration_ms=0, native_delay_ms=0, scene_owned=false},
                {row=25, selector="0x6352D26C", duration_ms=4173, native_delay_ms=0, scene_owned=false},
                {row=26, selector="0xD453DB47", duration_ms=1712, native_delay_ms=0, scene_owned=false},
                {row=27, selector="0xEB43430D", duration_ms=0, native_delay_ms=0, scene_owned=false},
                {row=28, selector="0xDC1E272E", duration_ms=0, native_delay_ms=0, scene_owned=false},
                {row=29, selector="0xB88C4BB2", duration_ms=3201, native_delay_ms=0, scene_owned=false},
                {row=30, selector="0x349D2672", duration_ms=5489, native_delay_ms=0, scene_owned=false},
                {row=31, selector="0x5F8E6160", duration_ms=2781, native_delay_ms=0, scene_owned=false},
                {row=32, selector="0x03622EEB", duration_ms=3529, native_delay_ms=0, scene_owned=false},
                {row=33, selector="0x921B35F9", duration_ms=3852, native_delay_ms=5000, scene_owned=false},
            },
            objective_cues={
                {row=18, objective="0x85A8F583"},
                {row=21, objective="0x85A8F583"},
                {row=31, objective="0x85A8F583"},
                {row=22, objective="0xA41DE99B"},
                {row=32, objective="0xA41DE99B"},
            },
        },
        cue_sets={
            landmarks={
                {event="lighthouse", cycles=array{}, actions={{operation="dialogue", value=0, delay_ms=0}}},
                {event="tunnel", cycles=array{}, actions={{operation="objective", value=515851809, delay_ms=0}, {operation="dialogue", value=6, delay_ms=1500}}},
                {event="forestVista", cycles=array{}, actions={{operation="objective", value=515851809, delay_ms=0}, {operation="dialogue", value=7, delay_ms=0}}},
                {event="forestExit", cycles=array{}, actions={{operation="objective", value=515851809, delay_ms=0}, {operation="dialogue", value=9, delay_ms=0}}},
                {event="lair", cycles=array{}, actions={{operation="objective", value=890754261, delay_ms=0}}},
                {event="arena", cycles=array{}, actions={{operation="objective", value=832904427, delay_ms=0}, {operation="dialogue", value=13, delay_ms=0}}},
            },
            encounters={
                {event="defenses", cycles={1, 2, 3}, actions={{operation="objective", value=832904427, delay_ms=0}}},
                {event="deletion", cycles={1}, actions={{operation="dialogue", value=14, delay_ms=0}}},
                {event="osirisArrives", cycles={1, 2, 3}, actions={{operation="dialogue", value=15, delay_ms=0}}},
                {event="osirisHolds", cycles={1}, actions={{operation="dialogue", value=16, delay_ms=0}}},
                {event="osirisHolds", cycles={2}, actions={{operation="dialogue", value=25, delay_ms=0}, {operation="dialogue", value=26, delay_ms=5840}}},
                {event="osirisHolds", cycles={3}, actions={{operation="dialogue", value=30, delay_ms=0}}},
                {event="arcReady", cycles={1, 2, 3}, actions={{operation="objective", value=2242442627, delay_ms=0}}},
                {event="arcReady", cycles={1}, actions={{operation="dialogue", value=18, delay_ms=0}}},
                {event="arcReminder", cycles={1, 2}, actions={{operation="dialogue", value=21, delay_ms=0}}},
                {event="arcReminder", cycles={3}, actions={{operation="dialogue", value=31, delay_ms=0}}},
                {event="eyeVulnerable", cycles={1, 2, 3}, actions={{operation="objective", value=2753423771, delay_ms=0}}},
                {event="eyeVulnerable", cycles={1}, actions={{operation="dialogue", value=22, delay_ms=0}}},
                {event="eyeVulnerable", cycles={3}, actions={{operation="dialogue", value=32, delay_ms=0}}},
                {event="pursuit", cycles={1, 2, 3}, actions={{operation="objective", value=3751228237, delay_ms=0}, {operation="dialogue", value=29, delay_ms=0}}},
                {event="defeated", cycles={1, 2, 3}, actions={{operation="dialogue", value=33, delay_ms=0}}},
            },
        },
        action_sets={
            reveal_complete={{operation="dialogue", value=12, delay_ms=250}},
            first_rescue_followup={{operation="dialogue", value=16, delay_ms=6400}},
        },
        binding_tables={
            forest={
                traversal={
                    {asset="forest/tunnel reached or passed/0", stage=1},
                    {asset="forest/vista reached or passed/0", stage=2},
                    {asset="forest/Forest exit reached or passed/0", stage=3},
                    {asset="forest/Lair reached or passed/0", stage=4},
                },
                objectives={
                    {asset="forest/tunnel presentation/1", event="0x1EBF4621"},
                    {asset="forest/Lair handoff/1", event="0x3517D4D5"},
                },
                dialogue={
                    {asset="forest/tunnel presentation/2", row=6, delay_ms=1500},
                    {asset="forest/vista presentation/2", row=7, delay_ms=0},
                    {asset="forest/Forest exit presentation/2", row=9, delay_ms=0},
                },
            },
        },
    },
}
