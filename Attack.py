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


class MultiAttack(object):

    def __init__(self):
        self.attacks = []

        self.damage_rv = None
        self.crit_damage_rv = None

        self.resisted_dmg_rv = None
        self.resisted_crit_dmg_rv = None

    def add_attack(self, atk):
        self.attacks.append(atk)

    def add_first_hit_damage(self, damage):
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

    def get_dmg_rv(self):
        all_dmg_rv = Constant(0)
        if self.damage_rv is not None or self.resisted_dmg_rv is not None:
            first_hit_outcomes = Outcomes()
            first_hit_outcomes.set_cap_lb(0)
            overall_outcomes = {}
            overall_outcomes[HitOutcome.MISS] = self.all_miss_()
            overall_outcomes[HitOutcome.HIT]  = self.first_hit_()
            overall_outcomes[HitOutcome.CRIT] = self.first_crit_()
            first_hit_outcomes.set_outcome_chances(overall_outcomes)

            damage_dict = {}
            damage_dict[HitOutcome.MISS] = Constant(0)
            if self.resisted_dmg_rv is None:
                damage_dict[HitOutcome.HIT] = self.damage_rv
            else:
                if self.damage_rv is None:
                    damage_dict[HitOutcome.HIT] = self.resisted_dmg_rv.half_round_down()
                else:
                    hit_dmg = self.damage_rv.add_rv(self.resisted_dmg_rv.half_round_down())
                    damage_dict[HitOutcome.HIT] = hit_dmg
            if self.resisted_crit_dmg_rv is None:
                damage_dict[HitOutcome.CRIT] = self.crit_damage_rv
            else:
                if self.crit_damage_rv is None:
                    damage_dict[HitOutcome.CRIT] = self.resisted_crit_dmg_rv.half_round_down()
                else:
                    crit_dmg = self.crit_damage_rv.add_rv(self.resisted_crit_dmg_rv.half_round_down())
                    damage_dict[HitOutcome.CRIT] = crit_dmg
            first_hit_outcomes.set_outcome_rvs(damage_dict)
            all_dmg_rv = first_hit_outcomes.get_rv()
        for atk in self.attacks:
            atk.finish_setup()
            atk_rv = atk.get_rv()
            all_dmg_rv = all_dmg_rv.add_rv(atk_rv)
        all_dmg_rv.memoize()
        return all_dmg_rv

    def all_miss_(self, index = 0):
        if index < len(self.attacks):
            value = self.attacks[index].get_outcome_chance(HitOutcome.MISS)
            return value * self.all_miss_(index+1)
        else:
            return 1

    def first_hit_(self, index = 0):
        if index < len(self.attacks):
            atk = self.attacks[0]
            hit_chance = atk.get_outcome_chance(HitOutcome.HIT)
            miss_chance = atk.get_outcome_chance(HitOutcome.MISS)
            return hit_chance + miss_chance * self.first_hit_(index+1)
        else:
            return 0

    def first_crit_(self, index = 0):
        if index < len(self.attacks):
            atk = self.attacks[0]
            crit_chance = atk.get_outcome_chance(HitOutcome.CRIT)
            miss_chance = atk.get_outcome_chance(HitOutcome.MISS)
            return crit_chance + miss_chance * self.first_crit_(index+1)
        else:
            return 0


class Attack(Outcomes):

    def __init__(self, bonus_rv, armor_class, hit_type = HitType.NORMAL, crit_lb = 20, halfling_lucky = False, auto_crit = False):
        super().__init__()
        self.bonuses = bonus_rv
        self.target = armor_class
        self.hit_type = hit_type
        self.crit_lb = crit_lb
        self.halfling_lucky = halfling_lucky
        self.auto_crit = auto_crit

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
        # damage is never negative
        self.set_cap_lb(0)

        self.damage_rv = None
        self.crit_damage_rv = None

        self.resisted_dmg_rv = None
        self.resisted_crit_dmg_rv = None

        self.is_setup_ = False

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
        if not self.is_setup_:
            if self.damage_rv is None and self.resisted_dmg_rv is None:
                raise RuntimeError("damage/resisted dmg RV is not set")
            damage_dict = {}
            damage_dict[HitOutcome.MISS] = Constant(0)
            if self.resisted_dmg_rv is None:
                damage_dict[HitOutcome.HIT] = self.damage_rv
            else:
                if self.damage_rv is None:
                    damage_dict[HitOutcome.HIT] = self.resisted_dmg_rv.half_round_down()
                else:
                    hit_dmg = self.damage_rv.add_rv(self.resisted_dmg_rv.half_round_down())
                    damage_dict[HitOutcome.HIT] = hit_dmg
            if self.resisted_crit_dmg_rv is None:
                damage_dict[HitOutcome.CRIT] = self.crit_damage_rv
            else:
                if self.crit_damage_rv is None:
                    damage_dict[HitOutcome.CRIT] = self.resisted_crit_dmg_rv.half_round_down()
                else:
                    crit_dmg = self.crit_damage_rv.add_rv(self.resisted_crit_dmg_rv.half_round_down())
                    damage_dict[HitOutcome.CRIT] = crit_dmg
            self.set_outcome_rvs(damage_dict)
            self.is_setup_ = True

    def get_ac(self):
        return self.target

    def get_hit_type(self):
        return self.hit_type

    def describe_attack(self):
        print("AC:", self.target)
        print("Hit Type:", self.hit_type)
        print()
        print("Outcomes RV:")
        self.describe_outcomes(True)
        attack_dmg_rv = self.get_rv()
        print()
        print("Attack RV:")
        attack_dmg_rv.describe(True)


