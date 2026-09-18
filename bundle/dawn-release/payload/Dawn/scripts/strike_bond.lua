-- A Garden World STRIKE, reconstructed from -uxfFqyMtxE and native package data.
-- The campaign scanner/Panoptes ending is outside this strike.
local composition=graph("composition","A Garden World",{step("mission",parallel("mission.module","mission.checked"))})
local conditions={
    condition("entered.tunnel",any_of("tunnel.tv_to_if_dialog","region.forest")),
    condition("forest.exit",any_of("forest.tv_end","region.past")),
    condition("past.gateway",any_of("forest.slot_001B","forest.tv_end","region.past")),
    condition("past.at_modules",any_of("past.tv_lower_cannon","past.slot_00ED","past.slot_00E9")),
    condition("past.terrace",any_of("past.slot_00EE","past.slot_00E8","past_dialogue.slot_000C")),
    condition("past.gates",any_of("past.slot_00E3","past.slot_00E4","past.slot_00E5","past_dialogue.slot_000A")),
    condition("spire.entry",any_of("spire_entry.slot_0002","spire_entry.slot_0003","region.spire")),
    condition("spire.dialogue_inside",any_of("spire_entry.tv_dialog_enter","spire.tv_entry")),
    condition("spire.mid",any_of("spire.tv_mid","spire.slot_0184","spire.slot_018B","spire.slot_0186")),
    condition("spire.top",any_of("spire_top.tv_top","arena.tv_arena")),
}
-- Helpers expand to immutable steps. Creation, scene start, destruction and
-- actor death remain separate observations. A request never proves completion.
local function add(t,id,commands,after)
    t[#t+1]=step(id,commands,{after=after or {}})
end
local function squads(t,id,prefix,first,last,after)
    local commands={}
    for i=first,last do commands[#commands+1]=prefix.."["..i.."].request" end
    add(t,id,commands,after)
end
local function block(t,id,prefix,after,unlock)
    add(t,id.."_create",parallel(prefix..".o_vex_lens.on",prefix..".o_wrapper.on",prefix..".o_vex_block.on",prefix..".d_vex_lens.on",prefix..".d_vex_block.on",prefix..".d_laser_in.on",prefix..".d_laser_out.on",prefix..".d_shield.on"),after)
    add(t,id.."_ready",prefix..".o_vex_lens.ready",{id.."_create"})
    add(t,id.."_expose",parallel(prefix..".o_vex_lens.expose",prefix..".o_vex_lens.marker"),unlock and {id.."_ready",unlock} or {id.."_ready"})
    add(t,id.."_destroyed",prefix..".o_vex_lens.destroyed",{id.."_expose"})
    add(t,id.."_open",parallel(prefix..".d_vex_block.off",prefix..".d_laser_in.off",prefix..".d_laser_out.off",prefix..".d_shield.off",prefix..".o_vex_block.off"),{id.."_destroyed"})
end
local function golem_setup(t,id,prefix,after,tether)
    add(t,id.."_create",parallel(prefix..".o_lens.on",prefix..".o_wrapper.on",prefix..".d_lens.on",prefix..".d_laser.on",prefix..(tether and ".d_shield.off" or ".d_shield.on")),after)
    add(t,id.."_scene",prefix..".sn_golem.start",{id.."_create"})
    add(t,id.."_started",parallel(prefix..".sn_golem.started",prefix..".sq_golem.ready"),{id.."_scene"})
    -- Beam preparation is asynchronous presentation. It must never hold the
    -- linked cube shielded or prevent the native Minotaur release sequence.
    if tether then
        -- Route guardians fight behind their cube-owned shield.
        -- Releasing AI does not finish the scene while its cube is intact.
        -- Neither cube exposure nor tether creation may wait for scene completion.
        add(t,id.."_awake",prefix..".sn_golem.release",{id.."_started"})
        add(t,id.."_tether",tether..".on",{id.."_started"})
    end
end
local function golem_fight(t,id,prefix,after,tether)
    add(t,id.."_expose",parallel(prefix..".o_lens.expose",prefix..".o_lens.marker"),after)
    add(t,id.."_destroyed",prefix..".o_lens.destroyed",{id.."_expose"})
    local release={prefix..".d_laser.off"}
    if not tether then release[#release+1]=prefix..".sn_golem.release" end
    release[#release+1]=tether and tether..".off" or prefix..".d_shield.off"
    add(t,id.."_release",release,{id.."_destroyed"})
    if not tether then add(t,id.."_released",prefix..".sn_golem.finished",{id.."_release"}) end
    add(t,id.."_killed",prefix..".sq_golem.cleared",{id..(tether and "_release" or "_released")})
end
local opening=graph("opening","The Lighthouse",{
    step("arrive","region.lighthouse"),
    step("defense",parallel("lighthouse.sq_top.request","lighthouse.sq_stairs_left.request","lighthouse.sq_stairs_right.request","lighthouse.sq_landing.request"),{after={"arrive"}}),
    step("briefing",parallel("objective.0","dialogue.0","portal.lighthouse_teleport.on"),{after={"arrive"}}),
    step("tunnel","entered.tunnel",{after={"briefing"}}),
    step("explanation",parallel("dialogue.1","objective.1","forest.generate"),{after={"tunnel"}}),
    step("leave","region.forest",{after={"explanation"}}),
})
local forest=graph("forest","The Infinite Forest",{
    step("generate",parallel("forest.generate","objective.1")),
    -- Forest C owns the procedural islands and Daemon doors. These are its
    -- fixed exit-platform sources, not substitute procedural populations.
    step("exit_defense",parallel("forest.sq_cyclops[0].request","forest.sq_cyclops[1].request","forest.sq_cyclops[2].request","forest.sq_goblins[0].request","forest.sq_goblins[1].request")),
    step("exit","forest.exit"),
    step("gate_approach","past.gateway"),
    step("objective","objective.2",{after={"gate_approach"}}),
    step("leave","region.past",{after={"objective"}}),
})
local p={}
add(p,"arrival_objective",parallel("objective.3","checkpoint.past"))
-- These four devices address the placed cannon effects, not the two separate
-- launch-source objects. Keep every route effect present before approaching it.
add(p,"cannon_effects",parallel("past.d_mancannon[0].on","past.d_mancannon[1].on","past.d_mancannon[2].on","past.d_mancannon[3].on"))
add(p,"mouth","past_dialogue.slot_000B")
add(p,"welcome","dialogue.3",{"mouth"})
squads(p,"arrival0","past.sq_arrival",0,7,{"welcome"})
add(p,"flanks",parallel("past.sq_arrival_right_flank.request","past.sq_arrival_left_flank.request","past.sq_lower_platform.request"),{"welcome"})
add(p,"lift","past.o_cannon.on",{"welcome"})
add(p,"cannon_approach","past.tv_lower_cannon",{"lift"})
add(p,"radiolaria",parallel("objective.8","dialogue.5"),{"cannon_approach"})
add(p,"modules","past.at_modules",{"lift"})
add(p,"security",parallel("dialogue.4","past.sq_upper_platform[0].request","past.sq_upper_platform[1].request"),{"modules"})
block(p,"block0","past.pf_block[0]",{"security"})
add(p,"barrier_power","dialogue.6",{"block0_ready"})
add(p,"path_revealed","dialogue.7",{"block0_open"})
add(p,"terrace","past.terrace",{"block0_open"})
add(p,"rebuilt","dialogue.8",{"terrace"})
squads(p,"terrace_high","past.sq_terrace_high",0,4,{"terrace"})
squads(p,"terrace_low","past.sq_terrace_low",0,3,{"terrace"})
block(p,"block1","past.pf_block[1]",{"terrace"})
local past=graph("past","Enter the Simulant Past",p)
local t={}
add(t,"presentation",parallel("objective.5","dialogue.9"))
squads(t,"support","past.sq_terrace_golem_support",0,3)
golem_setup(t,"terrace_golem","past.pf_terrace_golem",nil,"past.pf_anomaly[1].o_nomaly")
golem_fight(t,"terrace_golem","past.pf_terrace_golem",{"terrace_golem_tether"},"past.pf_anomaly[1].o_nomaly")
add(t,"teamwork","dialogue.10",{"terrace_golem_killed"})
block(t,"block2","past.pf_block[2]",nil,"terrace_golem_killed")
add(t,"corridor","past.gates",{"block2_open"})
local terrace=graph("terrace","Disable the first Minotaur shield",t)
local i={}
add(i,"security","objective.9")
add(i,"gate1",parallel("past.sq_gate1[0].request","past.sq_gate1[1].request","past.sq_gate1[2].request","past.sq_gate1_support[0].request","past.sq_gate1_support[1].request","past.sq_gate1_sniper[0].request","past.sq_gate1_sniper[1].request"))
block(i,"block3","past.pf_block[3]")
add(i,"gate2",parallel("past.sq_gate2[0].request","past.sq_gate2[1].request","past.sq_gate2[2].request","past.sq_gate2_support[0].request","past.sq_gate2_support[1].request","past.sq_gate2_support[2].request"),{"block3_open"})
block(i,"block4","past.pf_block[4]",{"block3_open"},"cannon_golem_killed")
add(i,"fifth_module","dialogue.11",{"block3_destroyed"})
add(i,"tower_guards",parallel("past.sq_cannon_snipers.request","past.sq_tower.request"),{"block4_destroyed"})
squads(i,"cannon_guards","past.sq_cannon",0,3,{"block3_open"})
golem_setup(i,"cannon_golem","past.pf_cannon_golem",{"block3_open"},"past.pf_anomaly[0].o_nomaly")
golem_fight(i,"cannon_golem","past.pf_cannon_golem",{"cannon_golem_tether"},"past.pf_anomaly[0].o_nomaly")
add(i,"cannon",parallel("objective.4","past.o_main_cannon.on","spire_entry.ap_to_machine.marker"),{"cannon_golem_killed","block4_open"})
add(i,"enter_spire","spire.entry",{"cannon"})
add(i,"dialogue_inside","spire.dialogue_inside",{"enter_spire"})
add(i,"arc_energy","dialogue.12",{"dialogue_inside"})
add(i,"leave","region.spire",{"arc_energy"})
local interior=graph("interior","Sabotage the remaining protocols",i)
local w={}
add(w,"presentation",parallel("objective.7","checkpoint.spire","spire.d_tower_laser.on"))
squads(w,"lower","spire.sq_lower",0,4)
-- All four placed effects belong to the climb, independently of the upper
-- floor's encounter. Preserve the existing traversal and combat dependencies.
add(w,"lower_cannon",parallel("spire.d_mancannon[0].on","spire.d_mancannon[1].on","spire.d_mancannon[2].on","spire.d_mancannon[3].on"))
add(w,"mid","spire.mid",{"lower_cannon"})
squads(w,"middle","spire.sq_mid",0,7,{"mid"})
add(w,"boss_prepare",parallel("spire.boss_platform.o_loop.on","spire.boss_platform.d.off","spire.d_laser_main.on","spire.sq_boss.request","spire.o_main_lens.on","spire.d_main_lens.on"),{"mid"})
add(w,"boss_ready",parallel("spire.sq_boss.ready","spire.o_main_lens.ready"),{"boss_prepare"})
add(w,"boss_intro","spire.sn_cyclops_intro.start",{"boss_ready"})
add(w,"boss_prepared","spire.sn_cyclops_intro.started",{"boss_intro"})
squads(w,"snipers","spire.sq_mid_sniper",0,4,{"mid"})
golem_setup(w,"tower_golem","spire.pf_tower_golem",{"mid"},"spire.pf_anomaly[0].o_nomaly")
golem_fight(w,"tower_golem","spire.pf_tower_golem",{"tower_golem_tether"},"spire.pf_anomaly[0].o_nomaly")
block(w,"tower_block","spire.pf_tower_block",{"mid"},"tower_golem_killed")
add(w,"upper_cannon","spire_top.slot_0002.marker",{"tower_golem_killed","tower_block_open","boss_prepared"})
squads(w,"top_guards","spire.sq_top",0,3,{"upper_cannon"})
add(w,"top","spire.top",{"upper_cannon"})
local tower=graph("tower","Climb the Spire",w)
local a={}
add(a,"arena","spire.top")
add(a,"presentation",parallel("objective.8","respawn.restrict","spire.o_main_lens.marker"),{"arena"})
add(a,"cover","arena.cover.start",{"opened"})
add(a,"boss_ready","spire.sq_boss.ready",{"arena"})
-- Spawn Dendron directly, then bind his native intro to the live boss and intact cube.
add(a,"lens_ready","spire.o_main_lens.ready",{"arena"})
add(a,"intro_started","spire.sn_cyclops_intro.started",{"lens_ready","boss_ready"})
add(a,"expose","spire.o_main_lens.expose",{"intro_started"})
add(a,"destroyed","spire.o_main_lens.destroyed",{"expose"})
add(a,"power_cut",parallel("dialogue.13","spire.d_laser_main.off","spire.d_tower_laser.off","boss.intro.exit"),{"destroyed"})
add(a,"intro_finished","spire.sn_cyclops_intro.finished",{"power_cut"})
add(a,"fight",parallel("objective.10","boss.fight"),{"intro_finished"})
add(a,"opened","boss.opened",{"fight"})
local intro=graph("intro","Cut the Spire Arc network",a)
local b={}
squads(b,"initial_adds","spire.sq_boss_adds",0,3)
squads(b,"lens_adds","spire.sq_lens_adds",0,5)
local function shield(t,n,first,second,after)
    local id="shield"..n
    local g0="spire.pf_golem["..first.."]"
    local g1="spire.pf_golem["..second.."]"
    add(t,id.."_health",parallel(n==1 and "boss.health.two_thirds" or "boss.health.one_third","boss.parked"..n),after)
    for i,g in ipairs({g0,g1}) do
        add(t,id.."_create"..i,parallel(g..".o_lens.on",g..".o_wrapper.on",g..".d_lens.on",g..".d_laser.on",g..".d_shield.on"),{id.."_health"})
    end
    add(t,id.."_scene",parallel(g0..".sn_golem.start",g1..".sn_golem.start","spire.d_laser_golem"..n..".on"),{id.."_create1",id.."_create2"})
    add(t,id.."_started",parallel(g0..".sn_golem.started",g1..".sn_golem.started",g0..".sq_golem.ready",g1..".sq_golem.ready"),{id.."_scene"})
    -- Like route guardians, these Minotaurs fight while their own cubes shield
    -- them. Release native AI on arrival without waiting for scene completion.
    add(t,id.."_expose",parallel(g0..".sn_golem.release",g1..".sn_golem.release",g0..".o_lens.expose",g1..".o_lens.expose"),{id.."_started"})
    for i,g in ipairs({g0,g1}) do
        add(t,id.."_destroyed"..i,g..".o_lens.destroyed",{id.."_expose"})
        add(t,id.."_release"..i,parallel(g..".d_laser.off",g..".d_shield.off",g..".sn_golem.finished"),{id.."_destroyed"..i})
        add(t,id.."_killed"..i,g..".sq_golem.cleared",{id.."_release"..i})
    end
    local finish={"boss.lens"..n..".destroyed","spire.d_laser_golem"..n..".off"}
    for i=(n==1 and 4 or 8),(n==1 and 7 or 11) do finish[#finish+1]="spire.sq_boss_adds["..i.."].request" end
    finish[#finish+1]="boss.awake"..n
    add(t,id.."_finished",finish,{id.."_killed1",id.."_killed2"})
    squads(t,id.."_adds","spire.sq_final_adds"..n,0,3,{id.."_started"})
end
shield(b,1,0,1)
shield(b,2,2,3,{"shield1_finished"})
squads(b,"final_adds","spire.sq_final_adds3",0,3,{"shield2_finished"})
add(b,"defeated","boss.dead",{"shield2_finished"})
local boss=graph("boss","Defeat Dendron, Root Mind",b)
-- A real boss death interrupts remaining shield/wave work. High damage can
-- legitimately end the fight early; native teardown is never counted as death.
local ending=graph("ending","Strike complete",{
    step("defeated","boss.dead"),
    step("reward",parallel("dialogue.14","spire.pf_boss_chest.o_chest.on","spire.d_laser_main.off","marker.clear","respawn.allow","arena.cover.stop","spire.d_laser_golem1.off","spire.d_laser_golem2.off"),{after={"defeated"}}),
    step("closing_dialogue","dialogue.14.finished",{after={"reward"}}),
    step("complete","mission.finish",{after={"closing_dialogue"}}),
})
return mission{
    id="strike_bond",graphs={composition,opening,forest,past,terrace,interior,tower,intro,boss,ending},
    roles={mission="composition",opening="opening",ending="ending"},
    entry="composition",modules={"mission"},observations={"mission.checked"},
    phases={"opening","forest","past","terrace","interior","tower","intro","boss"},conditions=conditions,
    presentation=presentation{markers={
        {objective="0x4E11E907",target="tunnel_portal"},
        {objective="0xF150883C",target="forest_gateway"},
        {objective="0x0864173F",target="forest_exit"},
        {objective="0x2D67AB51",target="spire_machine"},
        {objective="0x82271EEA",target="module0"},
        {objective="0x304AC854",target="module3"},
        {objective="0x6D15E881",target="terrace_golem_lens"},
        {objective="0x2A7ABBFB",target="past_cannon"},
        {objective="0x921AE90F",target="spire_ascend"},
        {objective="0xE6933775",target="dendron_lens"},
        {objective="0x665C1E1C",target="boss_arena"},
    }},
}
