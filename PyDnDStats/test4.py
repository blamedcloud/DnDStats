#!/usr/bin/python3

from MultiAttack import *
from Attack import *
from Enemy import *
from Damage import *

if __name__ == "__main__":

    # enemy assumptions
    armor_class = 14
    hit_type = HitType.NORMAL
    resisted = False
    auto_crit = False

    # lvl 3 rogue dual wielding shortswords, 16 dex, w/ sneak attack
    damage = Damage("1d6 + 3", resisted)
    offhand_damage = Damage("1d6", resisted)
    sneak_atk_dmg = Damage("2d6", resisted)

    hit_bonus = Constant(5) # dex + prof = 3 + 2

    enemy = Enemy(armor_class, hit_type, auto_crit)

    attack = Attack(hit_bonus, enemy)
    attack.add_damage(damage)

    offhand_atk = Attack(hit_bonus, enemy)
    offhand_atk.add_damage(offhand_damage)

    round_dmg = MultiAttack()
    round_dmg.add_attack(attack)
    round_dmg.add_attack(offhand_atk)
    round_dmg.add_first_hit_damage(sneak_atk_dmg)

    dpr = round_dmg.get_dmg_rv()

    dpr.describe(True)

