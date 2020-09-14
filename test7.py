#!/usr/bin/python3

from MultiAttack import *
from Attack import *
from Enemy import *
from Damage import *

if __name__ == "__main__":

    # enemy assumptions
    armor_class = 16
    hit_type = HitType.ADVANTAGE
    resisted = False
    auto_crit = False

    # lvl 9 half-orc zealot barbarian, reckless attack, greataxe
    damage = Damage("1d12 + 5 + 3", resisted)
    damage.set_crit_bonus("2d12")

    divine_fury_dmg = Damage("1d6 + 4", resisted)

    hit_bonus = Constant(9) # str + prof = 5 + 4

    enemy = Enemy(armor_class, hit_type, auto_crit)

    attack = Attack(hit_bonus, enemy)
    attack.add_damage(damage)

    attack2 = attack.copy()

    round_dmg = MultiAttack()
    round_dmg.add_attack(attack)
    round_dmg.add_attack(attack2)

    round_dmg.add_first_hit_damage(divine_fury_dmg)

    dpr = round_dmg.get_dmg_rv()

    dpr.describe(True)

