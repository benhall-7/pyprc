from pyprc import *

fighter_param = param("fighter_param.prc")
table = fighter_param[hash("fighter_param_table")]

mods = {
    hash("fighter_kind_pzenigame"): {
        hash("jump_count_max"): 2000,
        hash("landing_attack_air_frame_n"): 1,
        hash("landing_attack_air_frame_f"): 1,
        hash("landing_attack_air_frame_b"): 1,
        hash("landing_attack_air_frame_hi"): 1,
        hash("landing_attack_air_frame_lw"): 1,
    }
}

# actual param traversal and editing
for fighter in table:
    # the returned value is a hash, not a string
    fighter_name = fighter[hash("fighter_kind")].value

    if fighter_name in mods:
        fighter_mods = mods[fighter_name]
        for key in fighter_mods:
            fighter[key].value = fighter_mods[key]

fighter_param.save("fighter_param_new.prc")

