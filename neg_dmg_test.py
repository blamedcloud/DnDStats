#!/usr/bin/python3

from Common import *
from Attack import * 
from Enemy import *
from Damage import *

if __name__ == "__main__":

    damage = Damage("1d8 - 3")
    #damage = Damage("1d8")
    hit_bonus = Constant(5) # str + prof = 3 + 2
    # in the game you wouldn't have -3 to dmg and +5 to hit
    # but that isn't important here

    armor_class = 14
    hit_type = HitType.NORMAL
    resisted = False

    enemy = Enemy(armor_class, hit_type)
    damage.set_resisted(resisted)
    attack = Attack(hit_bonus, enemy)
    attack.add_damage(damage)
    attack.finish_setup()

    attack.describe_attack()
