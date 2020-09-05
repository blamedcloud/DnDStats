#!/usr/bin/python3

from MultiAttack import *
from Attack import *
from Damage import *

if __name__ == "__main__":

    # enemy assumptions
    armor_class = 17
    hit_type = HitType.NORMAL
    resisted = False
    auto_crit = False

    # lvl 11 gloomstalker ranger, rogue 1 using longbow, 20 dex, hunter's mark, sneak attack
    damage = Damage("1d8 + 1d6 + 5", resisted)
    bonus_atk_damage = Damage("2d8 + 1d6 + 5", resisted)
    sneak_atk_dmg = Damage("1d6", resisted)

    hit_bonus = Constant(11) # dex + prof + 2 = 5 + 4 + 2

    attack = Attack(hit_bonus, armor_class, hit_type, auto_crit = auto_crit)
    attack.add_damage(damage)

    attack2 = Attack(hit_bonus, armor_class, hit_type, auto_crit = auto_crit)
    attack2.add_damage(damage)

    attack3 = Attack(hit_bonus, armor_class, hit_type, auto_crit = auto_crit)
    attack3.add_damage(bonus_atk_damage)

    attack_miss = Attack(hit_bonus, armor_class, hit_type, auto_crit = auto_crit)
    attack_miss.add_damage(damage)

    round_dmg = MultiAttack()
    round_dmg.add_attack(attack)
    round_dmg.add_attack(attack2)
    round_dmg.add_attack(attack3)

    round_dmg.add_miss_extra_attack(attack_miss)

    round_dmg.add_first_hit_damage(sneak_atk_dmg)

    print("calculating round rv...")

    dpr = round_dmg.get_dmg_rv()

    dpr.describe(True)

