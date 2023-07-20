#!/usr/bin/python3

from MultiAttack import *
from Attack import *
from Enemy import *
from Damage import *
import math

if __name__ == "__main__":

    # assumptions
    armor_class = 14
    hit_type = HitType.SUPER_ADVANTAGE
    crit_lb = 19

    enemy = Enemy(armor_class, hit_type)

    sword_dmg = Damage("1d10r2 + 5 + 6")
    #sword_dmg = Damage("1d8 + 5 + 2")

    deadly_ambush_dmg = Damage("1d10r2 + 1d8 + 5 + 6")
    #deadly_ambush_dmg = Damage("2d8 + 5 + 2")

    eldritch_blast_dmg = Damage("1d10 + 5 + 6")

    cha_hit = Constant(11) # 5 cha + 6 prof

    sword_atk = Attack(cha_hit, enemy, crit_lb)
    sword_atk.add_damage(sword_dmg)

    deadly_ambush_atk = Attack(cha_hit, enemy, crit_lb)
    deadly_ambush_atk.add_damage(deadly_ambush_dmg)

    eldritch_blast_atk = Attack(cha_hit, enemy, crit_lb)
    eldritch_blast_atk.add_damage(eldritch_blast_dmg)

    sword_atks = 2
    deadly_ambush_atks = 2
    eldritch_blast_atks = 8

    round_dmg = MultiAttack()

    for atk in range(sword_atks):
        round_dmg.add_attack(sword_atk)

    for atk in range(deadly_ambush_atks):
        round_dmg.add_attack(deadly_ambush_atk)

    for atk in range(eldritch_blast_atks):
        round_dmg.add_attack(eldritch_blast_atk)

    print("enemy AC:",armor_class)
    print()

    dpr = round_dmg.get_dmg_rv()

    #dpr.describe(True, True, 1e-3)
    dpr.show_stats(True)
