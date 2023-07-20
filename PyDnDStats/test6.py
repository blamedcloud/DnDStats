#!/usr/bin/python3

from MultiAttack import *
from Attack import *
from Enemy import *
from Damage import *

if __name__ == "__main__":

    # enemy assumptions
    armor_class = 17
    hit_type = HitType.NORMAL
    resisted = False
    auto_crit = False

    turn_one = True

    # lvl 11 gloomstalker ranger, rogue 1 using longbow, 20 dex, hunter's mark, sneak attack
    damage = Damage("1d8 + 1d6 + 5", resisted)
    bonus_atk_damage = Damage("2d8 + 1d6 + 5", resisted)
    sneak_atk_dmg = Damage("1d6", resisted)

    hit_bonus = Constant(11) # dex + prof + 2 = 5 + 4 + 2

    enemy = Enemy(armor_class, hit_type, auto_crit)

    attack = Attack(hit_bonus, enemy)
    attack.add_damage(damage)

    attack2 = attack.copy()

    attack3 = Attack(hit_bonus, enemy)
    attack3.add_damage(bonus_atk_damage)

    attack_miss = attack.copy()

    round_dmg = MultiAttack()
    round_dmg.add_attack(attack)
    round_dmg.add_attack(attack2)

    if turn_one:
        round_dmg.add_attack(attack3)

    round_dmg.add_miss_extra_attack(attack_miss)

    round_dmg.add_first_hit_damage(sneak_atk_dmg)

    round_dmg2 = round_dmg.copy()

    dpr = round_dmg2.get_dmg_rv()

    dpr.describe(True)

