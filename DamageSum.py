#!/usr/bin/python3

from Common import *
from HitEnums import *

class DamageSum(object):

    def __init__(self):
        self.damage_rv = None
        self.crit_damage_rv = None

        self.resisted_dmg_rv = None
        self.resisted_crit_dmg_rv = None

    def has_damage(self):
        return self.damage_rv is not None or self.resisted_dmg_rv is not None

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
            self.resisted_dmg_rv.memoize()
            self.resisted_crit_dmg_rv.memoize()
        else:
            if self.damage_rv is None:
                self.damage_rv = damage.get_base_damage_rv()
            else:
                self.damage_rv = self.damage_rv.add_rv(damage.get_base_damage_rv())
            if self.crit_damage_rv is None:
                self.crit_damage_rv = damage.get_crit_damage_rv()
            else:
                self.crit_damage_rv = self.crit_damage_rv.add_rv(damage.get_crit_damage_rv())
            self.damage_rv.memoize()
            self.crit_damage_rv.memoize()

    def get_damage_dict(self):
        if not self.has_damage():
            raise RuntimeError("damage/resisted dmg RV is not set")
        damage_dict = {}
        for outcome in HitOutcome:
            damage_dict[outcome] = self.get_outcome_rv(outcome)
        return damage_dict

    def get_outcome_rv(self, outcome):
        if not self.has_damage():
            raise RuntimeError("damage/resisted dmg RV is not set")
        if outcome == HitOutcome.MISS:
            return Constant(0)
        elif outcome == HitOutcome.HIT:
            if self.resisted_dmg_rv is None:
                return self.damage_rv
            else:
                if self.damage_rv is None:
                    return self.resisted_dmg_rv.half_round_down()
                else:
                    return self.damage_rv.add_rv(self.resisted_dmg_rv.half_round_down())
        else: # outcome == HitOutcome.CRIT
            if self.resisted_crit_dmg_rv is None:
                return self.crit_damage_rv
            else:
                if self.crit_damage_rv is None:
                    return self.resisted_crit_dmg_rv.half_round_down()
                else:
                    return self.crit_damage_rv.add_rv(self.resisted_crit_dmg_rv.half_round_down())
