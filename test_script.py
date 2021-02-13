from pyprc import *

fighter_param = param("fighter_param.prc")
table = fighter_param["fighter_param_table"]

mods = {
    "pzenigame": {
        "jump_count_max": 8,
        "landing_attack_air_frame_n": 1,
        "landing_attack_air_frame_f": 1,
        "landing_attack_air_frame_b": 1,
        "landing_attack_air_frame_u": 1,
        "landing_attack_air_frame_d": 1,
    }
}

# convert fighter names to "fighter_kind_" hashes
mods_ = dict(mods)
for name in mods:
    mods_[hash("fighter_kind_" + name)] = mods[name]
mods = mods_

# actual param traversal and editing
for fighter in table:
    # the returned value is a hash, not a string
    fighter_name = fighter["fighter_kind"].value

    if fighter_name in mods:
        fighter_mods = mods[fighter_name]
        for key in fighter_mods:
            fighter[key].value = fighter_mods[key]

param.save("fighter_param_new.prc")

