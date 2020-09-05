#!/usr/bin/python3

from enum import Enum
from Outcomes import *
from Attack import *

class MultiAttack(object):

    def __init__(self):
        self.attacks = []

        self.damage_rv = None
        self.crit_damage_rv = None

        self.resisted_dmg_rv = None
        self.resisted_crit_dmg_rv = None

        self.damage_max = None
        self.damage_min_ub = None

        self.miss_atk = None
        self.miss_atk_ub = None

    def add_attack(self, atk):
        atk.finish_setup()
        self.attacks.append(atk)
        _, ub = atk.get_bounds()
        if self.damage_max is None:
            self.damage_max = ub
        else:
            self.damage_max += ub
        if self.damage_min_ub is None:
            self.damage_min_ub= ub
        elif ub < self.damage_min_ub:
            self.damage_min_ub = ub

    def add_miss_extra_attack(self, atk):
        atk.finish_setup()
        self.miss_atk = atk
        _, self.miss_atk_ub = atk.get_bounds()

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
            all_outcomes = self.generate_outcomes_product_()
            outcomes_rvs = {}
            outcomes_coeffs = {}
            for outcomes in all_outcomes:
                outcome_coeff = self.get_attack_outcome_chance_(outcomes)
                outcomes_coeffs[outcomes] = outcome_coeff

                attacks_rv = self.get_attack_outcome_rvs_(outcomes)
                fh_outcome = self.pick_first_passing_outcome_(outcomes, self.is_not_miss_)
                if fh_outcome is None:
                    fh_outcome = HitOutcome.MISS
                fhd_dmg_rv = self.get_fhd_dmg_rv_(fh_outcome)
                this_rv = attacks_rv.add_rv(fhd_dmg_rv)
                this_rv.memoize()
                outcomes_rvs[outcomes] = this_rv
            def overall_pdf(x):
                value = 0
                for outcomes, coeff in outcomes_coeffs.items():
                    value += coeff * outcomes_rvs[outcomes].pdf(x)
                return value
            # crit max damage *should* always be higher than hit max damage
            bonus_max = self.get_fhd_dmg_rv_(HitOutcome.CRIT).get_ub()
            all_dmg_rv = RandomVariable(0,self.damage_max + bonus_max)
            all_dmg_rv.set_pdf(overall_pdf)
        else:
            for atk in self.attacks:
                atk.finish_setup()
                atk_rv = atk.get_rv()
                all_dmg_rv = all_dmg_rv.add_rv(atk_rv)
        all_dmg_rv.memoize()
        return all_dmg_rv

    def get_attack_outcome_chance_(self, outcomes):
        product = 1
        assert(len(outcomes) == len(self.attacks))
        for i in range(len(outcomes)):
            product *= self.attacks[i].get_outcome_chance(outcomes[i])
        return product

    def get_fhd_dmg_rv_(self, outcome):
        if outcome == HitOutcome.MISS:
            return Constant(0)
        elif outcome == HitOutcome.HIT:
            if self.resisted_dmg_rv is None:
                if self.damage_rv is None:
                    return Constant(0)
                else:
                    return self.damage_rv
            else:
                if self.damage_rv is None:
                    return self.resisted_dmg_rv.half_round_down()
                else:
                    return self.damage_rv.add_rv(self.resisted_dmg_rv.half_round_down())
        else: # outcome == HitOutcome.CRIT
            if self.resisted_crit_dmg_rv is None:
                if self.crit_damage_rv is None:
                    return Constant(0)
                else:
                    return self.crit_damage_rv
            else:
                if self.crit_damage_rv is None:
                    return self.resisted_crit_dmg_rv.half_round_down()
                else:
                    return self.crit_damage_rv.add_rv(self.resisted_crit_dmg_rv.half_round_down())

    def get_attack_outcome_rvs_(self, outcomes):
        overall_rv = Constant(0)
        assert(len(outcomes) == len(self.attacks))
        for i in range(len(outcomes)):
            overall_rv = overall_rv.add_rv(self.attacks[i].get_outcome_rv(outcomes[i]))
        return overall_rv

    def is_not_miss_(self, outcome):
        return outcome != HitOutcome.MISS

    def is_miss_(self, outcome):
        return outcome == HitOutcome.MISS

    def pick_first_passing_outcome_(self, outcomes, criteria):
        assert(len(outcomes) == len(self.attacks))
        for i in range(len(outcomes)):
            if criteria(outcomes[i]):
                return outcomes[i]
        return None

    def generate_outcomes_product_(self, index = 0):
        if index == len(self.attacks) - 1:
            return [(outcome,) for outcome in self.attacks[index].get_outcomes()]
        else:
            this_outcomes = self.attacks[index].get_outcomes()
            later_outcomes = self.generate_outcomes_product_(index+1)
            return [(o1,) + o2 for o1 in this_outcomes for o2 in later_outcomes]


