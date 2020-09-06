#!/usr/bin/python3

from MultiAttack import *
from Attack import *
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

def sequoia_damage_haste(lvl, armor_class, sharpshooter = False, multitarget = False, haste_status = HasteStatus.INACTIVE, using_pw = False, hunters_mark_active = True):

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

    attack = Attack(hit_bonus, armor_class, HitType.NORMAL)
    if hunters_mark_active:
        attack.add_damage(longbow_hm)
    else:
        attack.add_damage(longbow)

    round_dmg = MultiAttack()

    for i in range(num_attacks):
        round_dmg.add_attack(attack.copy())

    if using_pw:
        round_dmg.add_first_hit_damage(planar_warrior)

    dmg = round_dmg.get_dmg_rv()

    return dmg

def haste_round(lvl, ac, sharpshooter, multitarget, round_num):
    if round_num == 1:
        # cast haste, use planar warrior
        return sequoia_damage_haste(lvl, ac, sharpshooter, multitarget, HasteStatus.CASTING, True, False)
    if round_num == 2:
        # cast hunter's mark (class feature variants, so no concentration)
        return sequoia_damage_haste(lvl, ac, sharpshooter, multitarget, HasteStatus.ACTIVE, False, True)
    else:
        # haste, hunter's mark active, using planar warrior
        return sequoia_damage_haste(lvl, ac, sharpshooter, multitarget, HasteStatus.ACTIVE, True, True)

def no_haste(lvl, ac, sharpshooter, multitarget, round_num):
    if round_num == 1:
        # cast hunter's mark
        return sequoia_damage_haste(lvl, ac, sharpshooter, multitarget, HasteStatus.INACTIVE, False, True)
    else:
        # hunter's mark active, using planar warrior
        return sequoia_damage_haste(lvl, ac, sharpshooter, multitarget, HasteStatus.INACTIVE, True, True)

def describe_combat(lvl, ac, sharpshooter, multitarget, total_rounds):
    dmg_func = no_haste
    if lvl >= 9:
        dmg_func = haste_round

    print("Sequoia level:",lvl)
    print("Enemy AC:",armor_class)
    print("Sharpshooter:",sharpshooter)

    overall = Constant(0)
    for round_num in range(1,combat_rounds+1):
        round_dmg = dmg_func(lvl, armor_class, sharpshooter, multitarget, round_num)
        print("Damage for round",round_num)
        round_dmg.show_stats()
        overall = overall.add_rv(round_dmg)
        overall.memoize()

    print("Overall Damage (across " + str(combat_rounds) + " rounds):")

    overall.show_stats()
    return overall

if __name__ == "__main__":

    armor_class = 15
    lvl = 8

    multitarget = False

    combat_rounds = 3

    describe_combat(lvl, armor_class, False, multitarget, combat_rounds)
    print()
    describe_combat(lvl, armor_class, True, multitarget, combat_rounds)



