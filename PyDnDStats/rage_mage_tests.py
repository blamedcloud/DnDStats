#!/usr/bin/python3

from MultiAttack import *
from Attack import *
from Enemy import *
from Damage import *
from enum import Enum
import math

class SpellStatus(Enum):
    CASTING = 0
    ACTIVE = 1
    DROPPED = 2
    INACTIVE = 3

class MultiClasses(Enum):
    BARBARIAN = 0
    WIZARD = 1
    ROGUE = 2
    FIGHTER = 3

def proficiency(lvl):
    return math.ceil(lvl/4)+1

def get_cantrip_dice(lvl):
    increases = [5,11,17]
    dice = 1
    for i in increases:
        if lvl >= i:
            dice += 1
    return dice

def get_character_lvl(class_lvls):
    char_lvl = 0
    for cls, lvl in class_lvls.items():
        char_lvl += lvl
    return char_lvl

def get_class_lvl(class_lvls, cls):
    if cls in class_lvls:
        return class_lvls[cls]
    else:
        return 0

def ragemage_damage(class_lvls, enemy, weapon_bonus, haste_status = SpellStatus.INACTIVE, shove = False, action_surge = False, giants_might = False, elven_accuracy = False, champion = False, greater_invis = SpellStatus.INACTIVE):

    lvl = get_character_lvl(class_lvls)
    prof = proficiency(lvl)
    cantrip_dice = get_cantrip_dice(lvl)

    rogue_lvl = get_class_lvl(class_lvls, MultiClasses.ROGUE)
    fighter_lvl = get_class_lvl(class_lvls, MultiClasses.FIGHTER)
    wizard_lvl = get_class_lvl(class_lvls, MultiClasses.WIZARD)
    barb_lvl = get_class_lvl(class_lvls, MultiClasses.BARBARIAN)

    sneak_attack_dice = math.ceil(rogue_lvl/2)
    first_hit_dice = sneak_attack_dice
    if fighter_lvl >= 3 and giants_might:
        first_hit_dice += 1
    first_hit_dmg = Damage(str(first_hit_dice) + "d6")

    dueling = 0
    if fighter_lvl > 0:
        dueling = 2

    damage_bonus = 5 + weapon_bonus + dueling
    rapier = Damage("1d8 + " + str(damage_bonus))

    booming_blade = rapier.copy()
    if cantrip_dice > 1:
        # one more d8 than the cantrip should do because of the base rapier damage
        booming_blade = Damage(str(cantrip_dice) + "d8 + " + str(damage_bonus))

    hit_bonus = Constant(5 + weapon_bonus + prof)

    num_attacks = 1
    num_attacks_bb = 0

    if wizard_lvl >= 6:
        num_attacks_bb = 1
    else:
        if fighter_lvl >= 5 or barb_lvl >= 5:
            num_attacks = 2

    if haste_status == SpellStatus.CASTING:
        num_attacks = 1
        num_attacks_bb = 0
    elif haste_status == SpellStatus.ACTIVE:
        num_attacks += 1
    elif haste_status == SpellStatus.DROPPED:
        num_attacks = 0
        num_attacks_bb = 0

    if greater_invis == SpellStatus.CASTING:
        num_attacks = 0
        num_attacks_bb = 0
        if elven_accuracy:
            enemy.apply_hit_type(HitType.SUPER_ADVANTAGE)
        else:
            enemy.apply_hit_type(HitType.ADVANTAGE)
    elif greater_invis == SpellStatus.ACTIVE:
        if elven_accuracy:
            enemy.apply_hit_type(HitType.SUPER_ADVANTAGE)
        else:
            enemy.apply_hit_type(HitType.ADVANTAGE)

    if action_surge and fighter_lvl >= 2:
        num_attacks += 1
        if wizard_lvl >= 6:
            num_attacks_bb += 1
        else:
            if fighter_lvl >=5 or barb_lvl >= 5:
                num_attacks += 1

    if shove and num_attacks + num_attacks_bb >= 2:
        num_attacks -= 1
        if elven_accuracy:
            enemy.apply_hit_type(HitType.SUPER_ADVANTAGE)
        else:
            enemy.apply_hit_type(HitType.ADVANTAGE)

    crit_range = 20
    if champion and fighter_lvl >= 3:
        crit_range = 19

    attack = Attack(hit_bonus, enemy, crit_range)
    attack.add_damage(rapier)

    attack_bb = Attack(hit_bonus, enemy, crit_range)
    attack_bb.add_damage(booming_blade)

    round_dmg = MultiAttack()

    for i in range(num_attacks):
        round_dmg.add_attack(attack.copy())

    for i in range(num_attacks_bb):
        round_dmg.add_attack(attack_bb.copy())

    round_dmg.add_first_hit_damage(first_hit_dmg)

    dmg = round_dmg.get_dmg_rv()
    return dmg

def haste_round(class_lvls, enemy, weapon_bonus, round_num, shove, action_surge, giants_might, elven_accuracy, champion):
    if round_num == 1:
        # activate blade song, cast haste
        return ragemage_damage(class_lvls, enemy, weapon_bonus, SpellStatus.CASTING, shove, action_surge, giants_might, elven_accuracy, champion)
    else:
        # active the other of blade song or giants might if applicable on round 2
        return ragemage_damage(class_lvls, enemy, weapon_bonus, SpellStatus.ACTIVE, shove, action_surge, giants_might, elven_accuracy, champion)

def no_haste_round(class_lvls, enemy, weapon_bonus, round_num, shove, action_surge, giants_might, elven_accuracy, champion):
    return ragemage_damage(class_lvls, enemy, weapon_bonus, SpellStatus.INACTIVE, shove, action_surge, giants_might, elven_accuracy, champion)

def greater_invis_round(class_lvls, enemy, weapon_bonus, round_num, shove, action_surge, giants_might, elven_accuracy, champion):
    if round_num == 1:
        # active blade song or giants might, cast haste
        return ragemage_damage(class_lvls, enemy, weapon_bonus, SpellStatus.INACTIVE, False, action_surge, giants_might, elven_accuracy, champion, SpellStatus.CASTING)
    else:
        # active the other of blade song or giants might if applicable on round 2
        return ragemage_damage(class_lvls, enemy, weapon_bonus, SpellStatus.INACTIVE, False, action_surge, giants_might, elven_accuracy, champion, SpellStatus.ACTIVE)


def describe_combat(class_lvls, enemy, weapon_bonus, total_rounds, use_haste = True, shoves = False, action_surge_turn = 1, use_giants_might = True, elven_accuracy = False, champion = False, use_gi = False):
    dmg_func = no_haste_round

    wizard_lvl = get_class_lvl(class_lvls, MultiClasses.WIZARD)
    if wizard_lvl >= 5 and use_haste:
        dmg_func = haste_round
    if wizard_lvl >= 7 and use_gi:
        dmg_func = greater_invis_round

    overall = Constant(0)
    for round_num in range(1,total_rounds+1):
        round_dmg = dmg_func(class_lvls, enemy.copy(), weapon_bonus, round_num, shoves, action_surge_turn == round_num, use_giants_might, elven_accuracy, champion)
        print("Damage for round",round_num)
        round_dmg.show_stats()
        overall = overall.add_rv(round_dmg)
        overall.memoize()

    print("Overall Damage (across " + str(total_rounds) + " rounds):")

    overall.show_stats()
    return overall

if __name__ == "__main__":

    #lvls
    class_lvls = {}
    class_lvls[MultiClasses.BARBARIAN] = 1
    class_lvls[MultiClasses.WIZARD] = 8
    class_lvls[MultiClasses.ROGUE] = 7
    class_lvls[MultiClasses.FIGHTER] = 4

    # assumptions
    armor_class = 17
    hit_type = HitType.NORMAL
    combat_rounds = 1
    weapon_bonus = 1
    elven_accuracy = True

    # resource usage
    use_haste = True
    action_surge_turn = 1
    use_giants_might = True
    is_champion = False
    use_greater_invis = False

    enemy = Enemy(armor_class, hit_type)

    print("Calculating Rune Knight damage with haste and shoves (always works)")
    dmg_shoves = describe_combat(class_lvls, enemy, weapon_bonus, combat_rounds, use_haste, True, action_surge_turn, use_giants_might, elven_accuracy, is_champion, use_greater_invis)
    print()
    print("Calculating Rune Knight damage with haste but no advantage source")
    dmg_no_shove = describe_combat(class_lvls, enemy, weapon_bonus, combat_rounds, use_haste, False, action_surge_turn, use_giants_might, elven_accuracy, is_champion, use_greater_invis)

    dmg_diff = dmg_shoves.subtract_rv(dmg_no_shove)

    print()
    print("Shove dmg - regular damage:")

    dmg_diff.show_stats()

    print("P(X > 0)", float(1 - dmg_diff.cdf(0)))

    # champion instead of rune knight
    use_haste = False
    action_surge_turn = 1
    use_giants_might = False
    is_champion = True
    use_greater_invis = True

    print("Calculating Champion damage with greater invis")
    dmg_gi = describe_combat(class_lvls, enemy, weapon_bonus, combat_rounds, use_haste, False, action_surge_turn, use_giants_might, elven_accuracy, is_champion, use_greater_invis)
