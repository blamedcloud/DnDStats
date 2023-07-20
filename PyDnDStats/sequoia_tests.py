#!/usr/bin/python3

from MultiAttack import *
from Attack import *
from Enemy import *
from Damage import *
from enum import Enum
import math

class HasteStatus(Enum):
    CASTING = 0
    ACTIVE = 1
    DROPPED = 2
    INACTIVE = 3

def proficiency(lvl):
    return math.ceil(lvl/4)+1

def sequoia_damage_haste(lvl, enemy, sharpshooter = False, multitarget = False, haste_status = HasteStatus.INACTIVE, using_pw = False, hunters_mark_active = True):

    # 6 is 20 dex and a +1 longbow
    longbow_hm = Damage("1d8 + 1d6 + 6")
    if sharpshooter:
        longbow_hm = Damage("1d8 + 1d6 + 16")

    longbow = Damage("1d8 + 6")
    if sharpshooter:
        longbow = Damage("1d8 + 16")

    planar_warrior = Damage("1d8")
    if lvl >= 11:
        planar_warrior = Damage("2d8")

    prof = proficiency(lvl)

    # 20 dex (+5), archery fighting style, +1 longbow
    hit_bonus = Constant(5 + 2 + 1 + prof)
    if sharpshooter:
        hit_bonus = Constant(2 + 1 + prof)

    num_attacks = 1
    num_hm_attacks = 0
    if hunters_mark_active:
        num_attacks = 0
        num_hm_attacks = 1
        if lvl >= 11 and multitarget:
            num_attacks = 2
            num_hm_attacks = 1
        elif lvl >= 5:
            num_hm_attacks = 2

        if haste_status == HasteStatus.CASTING:
            num_attacks = 0
            num_hm_attacks = 1
        elif haste_status == HasteStatus.ACTIVE:
            num_hm_attacks += 1
        elif haste_status == HasteStatus.DROPPED:
            num_attacks = 0
            num_hm_attacks = 0
    else:
        if lvl >= 11 and multitarget:
            num_attacks = 3
        elif lvl >= 5:
            num_attacks = 2

        if haste_status == HasteStatus.CASTING:
            num_attacks = 1
        elif haste_status == HasteStatus.ACTIVE:
            num_attacks += 1
        elif haste_status == HasteStatus.DROPPED:
            num_attacks = 0

    attack = Attack(hit_bonus, enemy)
    attack.add_damage(longbow)

    attack_hm = Attack(hit_bonus, enemy)
    attack_hm.add_damage(longbow_hm)

    round_dmg = MultiAttack()

    for i in range(num_attacks):
        round_dmg.add_attack(attack.copy())

    for i in range(num_hm_attacks):
        round_dmg.add_attack(attack_hm.copy())

    if using_pw:
        round_dmg.add_first_hit_damage(planar_warrior)

    dmg = round_dmg.get_dmg_rv()

    return dmg

def haste_round(lvl, enemy, sharpshooter, multitarget, round_num, use_hm, in_pw_range):
    if round_num == 1:
        # cast haste, use planar warrior
        return sequoia_damage_haste(lvl, enemy, sharpshooter, multitarget, HasteStatus.CASTING, in_pw_range, False)
    elif round_num == 2:
        # cast hunter's mark (class feature variants, so no concentration)
        return sequoia_damage_haste(lvl, enemy, sharpshooter, multitarget, HasteStatus.ACTIVE, (not use_hm) and in_pw_range, use_hm)
    else:
        # haste, hunter's mark active, using planar warrior
        return sequoia_damage_haste(lvl, enemy, sharpshooter, multitarget, HasteStatus.ACTIVE, in_pw_range, use_hm)

def no_haste(lvl, enemy, sharpshooter, multitarget, round_num, use_hm, in_pw_range):
    if round_num == 1:
        # cast hunter's mark
        return sequoia_damage_haste(lvl, enemy, sharpshooter, multitarget, HasteStatus.INACTIVE, (not use_hm) and in_pw_range, use_hm)
    else:
        # hunter's mark active, using planar warrior
        return sequoia_damage_haste(lvl, enemy, sharpshooter, multitarget, HasteStatus.INACTIVE, in_pw_range, use_hm)

def describe_combat(lvl, enemy, sharpshooter, multitarget, total_rounds, use_haste = True, use_hm = True, in_pw_range = True):
    dmg_func = no_haste
    if lvl >= 9 and use_haste:
        dmg_func = haste_round

    print("Sequoia level:",lvl)
    if enemy.get_ac().is_constant():
        print("Enemy AC:",enemy.get_ac().get_ub())
    print("Sharpshooter:",sharpshooter)

    overall = Constant(0)
    for round_num in range(1,total_rounds+1):
        round_dmg = dmg_func(lvl, enemy, sharpshooter, multitarget, round_num, use_hm, in_pw_range)
        print("Damage for round",round_num)
        round_dmg.show_stats()
        overall = overall.add_rv(round_dmg)
        overall.memoize()

    print("Overall Damage (across " + str(total_rounds) + " rounds):")

    overall.show_stats()
    return overall

if __name__ == "__main__":

    # assumptions
    armor_class = 18
    hit_type = HitType.NORMAL
    lvl = 10
    multitarget = False
    combat_rounds = 3
    use_haste = False
    use_hm = True
    in_pw_range = True

    enemy = Enemy(armor_class, hit_type)

    std_dmg = describe_combat(lvl, enemy, False, multitarget, combat_rounds, use_haste, use_hm, in_pw_range)
    print()
    ss_dmg  = describe_combat(lvl, enemy, True, multitarget, combat_rounds, use_haste, use_hm, in_pw_range)

    dmg_diff = ss_dmg.subtract_rv(std_dmg)

    print()
    print("Sharpshooter damage - regular damage:")

    dmg_diff.show_stats()

    print("P(X > 0):",float(1-dmg_diff.cdf(0)))


