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

    # lvl 12 gloomstalker ranger, rogue 1 using rapier, 20 dex, sneak attack
    # and Great Weapon Master (solely for the bonus crit atk opportunity)
    # this is obviously not optimal, but is a useful scenario for testing
    # (also has dueling fighting style)
    damage = Damage("1d8 + 5 + 2", resisted)
    bonus_atk_damage = Damage("2d8 + 5 + 2", resisted)
    sneak_atk_dmg = Damage("1d6", resisted)

    hit_bonus = Constant(10) # dex + prof = 5 + 5

    enemy = Enemy(armor_class, hit_type, auto_crit)

    attack = Attack(hit_bonus, enemy)
    attack.add_damage(damage)

    attack2 = attack.copy()

    attack3 = Attack(hit_bonus, enemy)
    attack3.add_damage(bonus_atk_damage)

    attack_miss = attack.copy()

    attack_crit = attack.copy()

    round_dmg = MultiAttack()
    round_dmg.add_attack(attack)
    round_dmg.add_attack(attack2)

    if turn_one:
        round_dmg.add_attack(attack3)

    round_dmg.add_miss_extra_attack(attack_miss)

    round_dmg.add_first_hit_damage(sneak_atk_dmg)

    round_dmg.add_crit_extra_attack(attack_crit)

    dpr = round_dmg.get_dmg_rv()

    dpr.describe(True)

