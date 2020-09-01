#!/usr/bin/python3

from Common import *
from Attack import * 
from Damage import *
from math import sqrt

if __name__ == "__main__":

    damage = Damage("2d6r2 + 3")
    hit_bonus = Constant(5) # str + prof = 3 + 2

    armor_class = 13
    hit_type = HitType.ADVANTAGE
    resisted = False

    print("AC:",armor_class)
    print("Hit Type:",hit_type)

    damage.set_resisted(resisted)
    attack = Attack(hit_bonus, armor_class, hit_type)
    attack.add_damage(damage)
    attack.finish_setup()

    print()
    print("Outcomes RV:")
    attack.describe_outcomes(True)

    attack_dmg_rv = attack.get_rv()

    print()
    print("Attack RV:")
    attack_dmg_rv.describe(True)
