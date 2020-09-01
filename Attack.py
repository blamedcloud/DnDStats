#!/usr/bin/python3

from RandomVariable import *
from Common import *
from enum import Enum
from Outcomes import *

class HitOutcome(Enum):
    MISS = 0
    HIT = 1
    CRIT = 2


class HitType(Enum):
    DISADVANTAGE = 0
    NORMAL = 1
    ADVANTAGE = 2
    SUPER_ADVANTAGE = 3


class Attack(Outcomes):

    def __init__(self, bonus_rv, armor_class, hit_type = HitType.NORMAL, crit_lb = 20, halfling_lucky = False):
        super().__init__()
        self.bonuses = bonus_rv
        self.target = armor_class
        self.hit_type = hit_type
        self.crit_lb = crit_lb
        self.halfling_lucky = halfling_lucky

        firstD20 = None

        if self.halfling_lucky:
            firstD20 = DiceReroll(20,1)
        else:
            firstD20 = Dice(20)

        overallD20 = None

        if self.hit_type == HitType.NORMAL:
            overallD20 = firstD20
        elif self.hit_type == HitType.DISADVANTAGE:
            overallD20 = firstD20.min_two_trials()
        elif self.hit_type == HitType.ADVANTAGE:
            overallD20 = firstD20.max_two_trials()
        else: # self.hit_type == HitType.SUPER_ADVANTAGE
            overallD20 = firstD20.max_three_trials()

        self.crit_chance = 1 - overallD20.cdf(self.crit_lb-1)
        self.auto_miss_chance = overallD20.pdf(1)

        hit_miss_d20 = RandomVariable(2,self.crit_lb-1)
        hit_miss_d20.set_pdf(overallD20.pdf)

        self.hit_miss_rv = hit_miss_d20.add_rv(self.bonuses)

        self.reg_miss_chance = self.hit_miss_rv.cdf(self.target-1)
        # can't use 1 - miss because some of the probability is tied up in crit_chance and auto_miss_chance
        self.hit_chance = self.hit_miss_rv.cdf(self.hit_miss_rv.get_ub()) - self.hit_miss_rv.cdf(self.target-1)

        chance_dict = {}
        chance_dict[HitOutcome.MISS] = self.auto_miss_chance + self.reg_miss_chance
        chance_dict[HitOutcome.HIT] = self.hit_chance
        chance_dict[HitOutcome.CRIT] = self.crit_chance

        self.set_outcome_chances(chance_dict)
        # damage is never negative
        self.set_cap_lb(0)

        self.damage_rv = None
        self.crit_damage_rv = None

        self.resisted_dmg_rv = None
        self.resisted_crit_dmg_rv = None

    def add_damage(self, damage):
        if damage.is_resisted():
            if self.resisted_dmg_rv is None:
                self.resisted_dmg_rv = damage.get_base_damage_rv()
            else:
                self.resisted_dmg_rv = self.resisted_dmg_rv.add_rv(damage.get_base_damage_rv())
            if self.resisted_crit_dmg_rv is None:
                self.resisted_crit_dmg_rv = damage.get_crit_damage_rv()
            else:
                self.resisted_crit_dmg_rv = self.resisted_crit_dmg_rv.add_rv(damage.get_crit_damage_rv())
        else:
            if self.damage_rv is None:
                self.damage_rv = damage.get_base_damage_rv()
            else:
                self.damage_rv = self.damage_rv.add_rv(damage.get_base_damage_rv())
            if self.crit_damage_rv is None:
                self.crit_damage_rv = damage.get_crit_damage_rv()
            else:
                self.crit_damage_rv = self.crit_damage_rv.add_rv(damage.get_crit_damage_rv())

    def finish_setup(self):
        if self.damage_rv is None or self.crit_damage_rv is None:
            raise RuntimeError("damage/crit RV is not set")
        damage_dict = {}
        damage_dict[HitOutcome.MISS] = Constant(0)
        if self.resisted_dmg_rv is None:
            damage_dict[HitOutcome.HIT] = self.damage_rv
        else:
            hit_dmg = self.damage_rv.add_rv(self.resisted_dmg_rv.half_round_down())
            damage_dict[HitOutcome.HIT] = hit_dmg
        if self.resisted_crit_dmg_rv is None:
            damage_dict[HitOutcome.CRIT] = self.crit_damage_rv
        else:
            crit_dmg = self.crit_damage_rv.add_rv(self.resisted_crit_dmg_rv.half_round_down())
            damage_dict[HitOutcome.CRIT] = crit_dmg
        self.set_outcome_rvs(damage_dict)


