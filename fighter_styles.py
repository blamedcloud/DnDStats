#!/usr/bin/python3

from MultiAttack import *
from Attack import *
from Enemy import *
from Damage import *
import math

def proficiency(lvl):
    return math.ceil(lvl/4)+1

def get_num_asis(lvl):
    asi_lvls = [4,6,8,12,14,16,19]
    asis = 0
    for asi_lvl in asi_lvls:
        if lvl >= asi_lvl:
            asis += 1
        else:
            break
    return asis

def get_num_attacks(lvl):
    extra_attacks = [5,11,20]
    attacks = 1
    for extra_atk in extra_attacks:
        if lvl >= extra_atk:
            attacks += 1
        else:
            break
    return attacks

def max_mod(start_mod, num_asis):
    final_mod = start_mod
    if start_mod < 5:
        if num_asis + start_mod >= 5:
            diff = 5 - start_mod
            num_asis -= diff
            final_mod = 5
        else:
            final_mod += num_asis
            num_asis = 0
    return (final_mod, num_asis)

def archery_damage(lvl, start_mod, weapon_bonus, enemy, power_atk):

    num_asis = get_num_asis(lvl)
    final_mod, num_asis = max_mod(start_mod, num_asis)
    action_attacks = get_num_attacks(lvl)
    prof = proficiency(lvl)

    sharpshooter = False
    if num_asis >= 1:
        sharpshooter = True

    damage_mod = final_mod + weapon_bonus
    weapon_dmg = Damage("1d8 + " + str(damage_mod))
    if sharpshooter and power_atk:
        weapon_dmg = Damage("1d8 + " + str(damage_mod + 10))

    to_hit = final_mod + 2 + weapon_bonus + prof
    hit_bonus = Constant(to_hit)
    if sharpshooter and power_atk:
        hit_bonus = Constant(to_hit - 5)

    attack = Attack(hit_bonus, enemy)
    attack.add_damage(weapon_dmg)

    round_dmg = MultiAttack()

    for i in range(action_attacks):
        round_dmg.add_attack(attack.copy())

    dmg = round_dmg.get_dmg_rv()

    return dmg

def longsword_damage(lvl, start_mod, weapon_bonus, enemy, power_atk):

    num_asis = get_num_asis(lvl)
    final_mod, num_asis = max_mod(start_mod, num_asis)
    action_attacks = get_num_attacks(lvl)
    prof = proficiency(lvl)

    if num_asis >= 1:
        # get shield master, assume the shove knocks prone always
        # this isn't the best assumption, but I don't have a way
        # to handle this conditional yet.
        enemy.apply_hit_type(HitType.ADVANTAGE)

    damage_mod = final_mod + weapon_bonus + 2
    weapon_dmg = Damage("1d8 + " + str(damage_mod))

    to_hit = final_mod + weapon_bonus + prof
    hit_bonus = Constant(to_hit)

    attack = Attack(hit_bonus, enemy)
    attack.add_damage(weapon_dmg)

    round_dmg = MultiAttack()

    for i in range(action_attacks):
        round_dmg.add_attack(attack.copy())

    dmg = round_dmg.get_dmg_rv()

    return dmg

def greatsword_damage(lvl, start_mod, weapon_bonus, enemy, power_atk):

    num_asis = get_num_asis(lvl)
    final_mod, num_asis = max_mod(start_mod, num_asis)
    action_attacks = get_num_attacks(lvl)
    prof = proficiency(lvl)

    gwm = False
    if num_asis >= 1:
        gwm = True

    damage_mod = final_mod + weapon_bonus
    weapon_dmg = Damage("2d6r2 + " + str(damage_mod))
    if gwm and power_atk:
        weapon_dmg = Damage("2d6r2 + " + str(damage_mod + 10))

    to_hit = final_mod + weapon_bonus + prof
    hit_bonus = Constant(to_hit)
    if gwm and power_atk:
        hit_bonus = Constant(to_hit - 5)

    attack = Attack(hit_bonus, enemy)
    attack.add_damage(weapon_dmg)

    round_dmg = MultiAttack()

    for i in range(action_attacks):
        round_dmg.add_attack(attack.copy())

    if gwm:
        round_dmg.add_crit_extra_attack(attack.copy())

    dmg = round_dmg.get_dmg_rv()

    return dmg

def halberd_damage(lvl, start_mod, weapon_bonus, enemy, power_atk):

    num_asis = get_num_asis(lvl)
    final_mod, num_asis = max_mod(start_mod, num_asis)
    action_attacks = get_num_attacks(lvl)
    prof = proficiency(lvl)

    gwm = False
    if num_asis >= 1:
        gwm = True
        num_asis -= 1

    pam = False
    if num_asis >= 1:
        pam = True

    damage_mod = final_mod + weapon_bonus
    weapon_dmg = Damage("1d10r2 + " + str(damage_mod))
    pommel_dmg = Damage("1d4r2 + " + str(damage_mod))
    if gwm and power_atk:
        weapon_dmg = Damage("1d10r2 + " + str(damage_mod + 10))
        pommel_dmg = Damage("1d4r2 + " + str(damage_mod + 10))

    to_hit = final_mod + weapon_bonus + prof
    hit_bonus = Constant(to_hit)
    if gwm and power_atk:
        hit_bonus = Constant(to_hit - 5)

    attack = Attack(hit_bonus, enemy)
    attack.add_damage(weapon_dmg)

    round_dmg = MultiAttack()

    for i in range(action_attacks):
        round_dmg.add_attack(attack.copy())

    # technically, you can wait to use the pam bonus action
    # until you don't crit your normal attacks, otherwise you
    # get a 1d10 bonus action attack from gwm rather than 1d4
    # however, that is much more complicated to simulate
    # and unlikely to change the result much
    if pam:
        pommel_atk = Attack(hit_bonus, enemy)
        pommel_atk.add_damage(pommel_dmg)

        round_dmg.add_attack(pommel_atk)
    elif gwm:
        round_dmg.add_crit_extra_attack(attack.copy())

    dmg = round_dmg.get_dmg_rv()

    return dmg

def dual_wield_damage(lvl, start_mod, weapon_bonus, enemy, power_atk):

    num_asis = get_num_asis(lvl)
    final_mod, num_asis = max_mod(start_mod, num_asis)
    action_attacks = get_num_attacks(lvl)
    prof = proficiency(lvl)

    weapon_die = "1d6"
    if num_asis >= 1:
        # dual wielder feat (sigh...)
        weapon_die = "1d8"

    damage_mod = final_mod + weapon_bonus
    weapon_dmg = Damage(weapon_die + " + " + str(damage_mod))

    to_hit = final_mod + weapon_bonus + prof
    hit_bonus = Constant(to_hit)

    attack = Attack(hit_bonus, enemy)
    attack.add_damage(weapon_dmg)

    round_dmg = MultiAttack()

    bonus_attacks = 1
    #bonus_attacks = action_attacks

    for i in range(action_attacks+bonus_attacks):
        round_dmg.add_attack(attack.copy())

    dmg = round_dmg.get_dmg_rv()

    return dmg

if __name__ == "__main__":

    # assumptions
    armor_class = 13
    hit_type = HitType.NORMAL
    lvl = 4
    start_mod = 3
    weapon_mod = 0
    power_atk = True

    enemy = Enemy(armor_class, hit_type)

    print("lvl:",lvl)
    print("enemy AC:",armor_class)
    print()

    print("Archery style:")
    archery_dmg = archery_damage(lvl, start_mod, weapon_mod, enemy, power_atk)
    archery_dmg.show_stats()
    print()

    print("Greatsword style:")
    greatsword_dmg = greatsword_damage(lvl, start_mod, weapon_mod, enemy, power_atk)
    greatsword_dmg.show_stats()
    print()

    print("Halberd style:")
    halberd_dmg = halberd_damage(lvl, start_mod, weapon_mod, enemy, power_atk)
    halberd_dmg.show_stats()
    print()

    print("Sword+Board style:")
    longsword_dmg = longsword_damage(lvl, start_mod, weapon_mod, enemy, power_atk)
    longsword_dmg.show_stats()
    print()

    print("Dual Wield style:")
    dual_wield_dmg = dual_wield_damage(lvl, start_mod, weapon_mod, enemy, power_atk)
    dual_wield_dmg.show_stats()
    print()
