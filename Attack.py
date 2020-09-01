#!/usr/bin/python3

from RandomVariable import *
from Common import *
from enum import Enum

class HitOutcome(Enum):
    MISS = 0
    HIT = 1
    CRIT = 2


class HitType(Enum):
    DISADVANTAGE = 0
    NORMAL = 1
    ADVANTAGE = 2
    SUPER_ADVANTAGE = 3


class Attack(object):

    def __init__(self, bonus_rv, armor_class, hit_type = HitType.NORMAL, crit_lb = 20, halfling_lucky = False):
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

        self.damage_rv = None
        self.crit_damage_rv = None
        
    def set_damage_rv(self, damage):
        self.damage_rv = damage

    def set_crit_bonus_rv(self, damage):
        if self.damage_rv is None:
            raise RuntimeError("damage RV is not set")
        self.crit_damage_rv = self.damage_rv.add_rv(damage)

    def describe_outcomes(self, approx = False):
        total = 0
        for outcome in HitOutcome:
            outcome_chance = self.get_outcome_chance(outcome)
            total += outcome_chance
            if approx:
                print(outcome,":",outcome_chance,"~=",float(outcome_chance))
            else:
                print(outcome,":",outcome_chance)
        print("Total:",total)

    def get_outcome_chance(self, outcome):
        if outcome == HitOutcome.MISS:
            return self.get_miss_chance()
        elif outcome == HitOutcome.HIT:
            return self.get_hit_chance()
        else: # outcome == HitOutcome.CRIT
            return self.get_crit_chance()

    def get_miss_chance(self):
        return self.auto_miss_chance + self.reg_miss_chance

    def get_hit_chance(self):
        return self.hit_chance

    def get_crit_chance(self):
        return self.crit_chance

    def get_attack_rv(self):
        def dmg_pdf(x):
            if x == 0:
                # cdf instead of pdf because negative damage counts as 0 damage
                return self.get_miss_chance() + self.get_hit_chance() * self.damage_rv.cdf(0) + self.get_crit_chance() * self.crit_damage_rv.cdf(0)
            else:
                return self.get_hit_chance() * self.damage_rv.pdf(x) + self.get_crit_chance() * self.crit_damage_rv.pdf(x)
        true_ub = max(self.damage_rv.get_ub(),self.crit_damage_rv.get_ub())
        attackRV = RandomVariable(0,true_ub)
        attackRV.set_pdf(dmg_pdf)
        return attackRV


