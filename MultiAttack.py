#!/usr/bin/python3

from Common import *
from HitEnums import *
from DamageSum import *

class MultiAttack(object):

    def __init__(self):
        self.attacks = []

        self.first_hit_damage = DamageSum()

        self.damage_max = None
        self.damage_min_ub = None

        self.miss_atk = None
        self.miss_atk_ub = None

        self.crit_atk = None
        self.crit_atk_ub = None

        self.debug = False

    def copy(self):
        multi = MultiAttack()
        for atk in self.attacks:
            multi.add_attack(atk.copy())
        multi.first_hit_damage = self.first_hit_damage.copy()
        if self.miss_atk is not None:
            multi.add_miss_extra_attack(self.miss_atk.copy())
        multi.set_debug(self.debug)
        return multi

    def set_debug(self, debug):
        self.debug = debug

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

    def add_crit_extra_attack(self, atk):
        atk.finish_setup()
        self.crit_atk = atk
        _, self.crit_atk_ub = atk.get_bounds()

    def add_first_hit_damage(self, damage):
        self.first_hit_damage.add_damage(damage)

    def has_bonus_atk(self):
        return self.miss_atk is not None or self.crit_atk is not None

    def get_dmg_rv(self):
        all_dmg_rv = Constant(0)
        if self.first_hit_damage.has_damage() or self.has_bonus_atk():
            all_outcomes = self.generate_outcomes_product_()
            outcomes_rvs = {}
            outcomes_coeffs = {}
            if self.debug:
                print("number of outcomes:",len(all_outcomes))
            for i, outcomes in enumerate(all_outcomes):
                if self.debug:
                    print("outcome number:",i)
                outcome_coeff = self.get_attack_outcome_chance_(outcomes)
                attacks_rv = self.get_attack_outcome_rvs_(outcomes)

                fm_outcome = self.pick_first_passing_outcome_(outcomes, self.is_miss_)
                if fm_outcome is not None and self.miss_atk is not None:
                    miss_atk_outcomes = self.miss_atk.get_outcomes()
                    for outcome in miss_atk_outcomes:
                        new_outcomes = outcomes + (outcome,)
                        new_coeff = outcome_coeff * self.miss_atk.get_outcome_chance(outcome)
                        new_rv = attacks_rv.add_rv(self.miss_atk.get_outcome_rv(outcome))
                        new_rv.memoize()

                        fc_outcome = self.pick_first_passing_outcome_(new_outcomes, self.is_crit_)
                        if fc_outcome is not None and self.crit_atk is not None:
                            crit_atk_outcomes = self.crit_atk.get_outcomes()
                            for outcome_c in crit_atk_outcomes:
                                new_c_outcomes = new_outcomes + (outcome_c,)
                                new_c_coeff = new_coeff * self.crit_atk.get_outcome_chance(outcome_c)
                                new_c_rv = new_rv.add_rv(self.crit_atk.get_outcome_rv(outcome_c))
                                new_c_rv.memoize()
                                new_c_rv = self.handle_fh_outcome_(new_c_outcomes, new_c_rv)
                                outcomes_coeffs[new_c_outcomes] = new_c_coeff
                                outcomes_rvs[new_c_outcomes] = new_c_rv
                        else:
                            new_rv = self.handle_fh_outcome_(new_outcomes, new_rv)
                            outcomes_coeffs[new_outcomes] = new_coeff
                            outcomes_rvs[new_outcomes] = new_rv
                else:
                    fc_outcome = self.pick_first_passing_outcome_(outcomes, self.is_crit_)
                    if fc_outcome is not None and self.crit_atk is not None:
                        crit_atk_outcomes = self.crit_atk.get_outcomes()
                        for outcome in crit_atk_outcomes:
                            new_outcomes = outcomes + (outcome,)
                            new_coeff = outcome_coeff * self.crit_atk.get_outcome_chance(outcome)
                            new_rv = attacks_rv.add_rv(self.crit_atk.get_outcome_rv(outcome))
                            new_rv.memoize()

                            fm_outcome = self.pick_first_passing_outcome_(new_outcomes, self.is_miss_)
                            if fm_outcome is not None and self.miss_atk is not None:
                                miss_atk_outcomes = self.miss_atk.get_outcomes()
                                for outcome_m in miss_atk_outcomes:
                                    new_m_outcomes = new_outcomes + (outcome_m,)
                                    new_m_coeff = new_coeff * self.miss_atk.get_outcome_chance(outcome_m)
                                    new_m_rv = new_rv.add_rv(self.miss_atk.get_outcome_rv(outcome_m))
                                    new_m_rv.memoize()
                                    new_m_rv = self.handle_fh_outcome_(new_m_outcomes, new_m_rv)
                                    outcomes_coeffs[new_m_outcomes] = new_m_coeff
                                    outcomes_rvs[new_m_outcomes] = new_m_rv
                            else:
                                new_rv = self.handle_fh_outcome_(new_outcomes, new_rv)
                                outcomes_coeffs[new_outcomes] = new_coeff
                                outcomes_rvs[new_outcomes] = new_rv
                    else:
                        attacks_rv = self.handle_fh_outcome_(outcomes, attacks_rv)
                        outcomes_coeffs[outcomes] = outcome_coeff
                        outcomes_rvs[outcomes] = attacks_rv
            def overall_pdf(x):
                value = 0
                for outcomes, coeff in outcomes_coeffs.items():
                    value += coeff * outcomes_rvs[outcomes].pdf(x)
                return value
            # crit max damage *should* always be higher than hit max damage
            bonus_max = self.get_fhd_dmg_rv_(HitOutcome.CRIT).get_ub()
            if self.miss_atk_ub is not None:
                bonus_max += self.miss_atk_ub - self.damage_min_ub
            if self.crit_atk_ub is not None:
                bonus_max += self.crit_atk_ub
            all_dmg_rv = RandomVariable(0,self.damage_max + bonus_max)
            all_dmg_rv.set_pdf(overall_pdf)
        else:
            for atk in self.attacks:
                atk.finish_setup()
                atk_rv = atk.get_rv()
                all_dmg_rv = all_dmg_rv.add_rv(atk_rv)
                all_dmg_rv.memoize()
        all_dmg_rv.memoize()
        return all_dmg_rv

    def handle_fh_outcome_(self, outcomes, input_rv):
        fh_outcome = self.pick_first_passing_outcome_(outcomes, self.is_not_miss_)
        if fh_outcome is not None:
            fh_rv = self.get_fhd_dmg_rv_(fh_outcome)
            input_rv = input_rv.add_rv(fh_rv)
            input_rv.memoize()
        return input_rv

    def get_attack_outcome_chance_(self, outcomes):
        product = 1
        assert(len(outcomes) == len(self.attacks))
        for i in range(len(outcomes)):
            product *= self.attacks[i].get_outcome_chance(outcomes[i])
        return product

    def get_fhd_dmg_rv_(self, outcome):
        if self.first_hit_damage.has_damage():
            return self.first_hit_damage.get_outcome_rv(outcome)
        else:
            return Constant(0)

    def get_attack_outcome_rvs_(self, outcomes):
        overall_rv = Constant(0)
        assert(len(outcomes) == len(self.attacks))
        for i in range(len(outcomes)):
            overall_rv = overall_rv.add_rv(self.attacks[i].get_outcome_rv(outcomes[i]))
            overall_rv.memoize()
        return overall_rv

    def is_not_miss_(self, outcome):
        return outcome != HitOutcome.MISS

    def is_miss_(self, outcome):
        return outcome == HitOutcome.MISS

    def is_crit_(self, outcome):
        return outcome == HitOutcome.CRIT

    def pick_first_passing_outcome_(self, outcomes, criteria):
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


