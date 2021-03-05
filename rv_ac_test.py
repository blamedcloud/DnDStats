#!/usr/bin/python3

from Common import *
from Attack import * 
from Enemy import *
from Damage import *

if __name__ == "__main__":

    damage = Damage("1d4 + 3")
    hit_bonus = Constant(5) # str + prof = 3 + 2

    armor_class = Uniform(10,13)
    hit_type = HitType.ADVANTAGE
    resisted = False

    enemy = Enemy(armor_class, hit_type)
    damage.set_resisted(resisted)
    attack = Attack(hit_bonus, enemy)
    attack.add_damage(damage)
    attack.finish_setup()

    attack.describe_attack()
