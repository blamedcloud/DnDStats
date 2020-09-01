#!/usr/bin/python3

from Common import *
from Attack import * 
from math import sqrt

if __name__ == "__main__":

    d6 = DiceReroll(6,2)
    constant = Constant(3)
    damage = d6.add_rv(constant)
    damage = damage.add_rv(d6)
    #damage.show_pdf()
    #print(damage.expected_value())

    crit_dmg = d6.add_rv(d6)
    
    hit_bonus = Constant(5) # str + prof = 3 + 2

    armor_class = 13
    hit_type = HitType.ADVANTAGE
    resisted = False

    print("AC:",armor_class)
    print("Hit Type:",hit_type)

    if resisted:
        damage = damage.half_round_down()
        crit_dmg = crit_dmg.half_round_down()

    attack = Attack(hit_bonus, armor_class, hit_type)
    attack.set_damage_rv(damage)
    attack.set_crit_bonus_rv(crit_dmg)
    attack.finish_setup()

    print()
    print("Outcomes RV:")
    attack.describe_outcomes(True)

    attack_dmg_rv = attack.get_rv()

    print()
    print("Attack RV:")
    attack_dmg_rv.describe(True)
