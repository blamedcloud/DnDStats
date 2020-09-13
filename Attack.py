#!/usr/bin/python3

from RandomVariable import *
from OutcomeRV import *
from HitEnums import *
from DamageSum import *

class Attack(OutcomeRV):

    def __init__(self, bonus_rv, armor_class, hit_type = HitType.NORMAL, crit_lb = 20, halfling_lucky = False, auto_crit = False):
        super().__init__()
        self.bonuses = bonus_rv
        self.target = armor_class
        self.hit_type = hit_type
        self.crit_lb = crit_lb
        self.halfling_lucky = halfling_lucky
        self.auto_crit = auto_crit

        # damage is never negative
        self.set_cap_lb(0)

        self.damage_sum = DamageSum()

        self.is_setup_ = False

    def copy(self):
        atk = Attack(self.bonuses.copy(), self.target, self.hit_type, self.crit_lb, self.halfling_lucky, self.auto_crit)
        atk.damage_sum = self.damage_sum.copy()
        if self.is_setup_:
            atk.finish_setup()
        return atk

    def add_damage(self, damage):
        self.damage_sum.add_damage(damage)

    def setup_damage_(self):
        damage_dict = self.damage_sum.get_damage_dict()
        self.set_outcome_rvs(damage_dict)

    def setup_chances_(self):
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
        if self.auto_crit:
            chance_dict[HitOutcome.HIT] = 0
            chance_dict[HitOutcome.CRIT] = self.hit_chance + self.crit_chance
        else:
            chance_dict[HitOutcome.HIT] = self.hit_chance
            chance_dict[HitOutcome.CRIT] = self.crit_chance
        self.set_outcome_chances(chance_dict)

    def finish_setup(self):
        if not self.is_setup_:
            self.setup_chances_()
            self.setup_damage_()
            self.is_setup_ = True

    def get_ac(self):
        return self.target

    def get_hit_type(self):
        return self.hit_type

    def describe_attack(self):
        print("AC:", self.target)
        print("Hit Type:", self.hit_type)
        print()
        print("Outcome RV:")
        self.describe_outcomes(True)
        attack_dmg_rv = self.get_rv()
        print()
        print("Attack RV:")
        attack_dmg_rv.describe(True)


