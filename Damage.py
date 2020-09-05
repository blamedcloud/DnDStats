#!/usr/bin/python3

from RandomVariable import *
from Common import *

class Damage(object):

    def __init__(self, dmg_expr = None, resisted = False):
        self.damage_rv = None
        self.crit_rv = None

        self.static_dmg_key = "static"
        self.resisted = resisted

        if dmg_expr is not None:
            self.set_damage(dmg_expr)

    def is_resisted(self):
        return self.resisted

    def get_base_damage_rv(self):
        return self.damage_rv

    def get_crit_damage_rv(self):
        return self.crit_rv

    def set_damage(self, dmg_expr):
        damage_dict = self.parse_dmg_expr_(dmg_expr)
        static_dmg = Constant(0)
        if self.static_dmg_key in damage_dict:
            static_dmg = damage_dict[self.static_dmg_key]
            del damage_dict[self.static_dmg_key]
        other_dmg = None
        for _, dmg in damage_dict.items():
            if other_dmg is None:
                other_dmg = dmg
            else:
                other_dmg = other_dmg.add_rv(dmg)
        non_static_damage = other_dmg
        crit_damage = other_dmg.add_rv(other_dmg)

        self.damage_rv = non_static_damage.add_rv(static_dmg)
        self.crit_rv = crit_damage.add_rv(static_dmg)

        self.damage_rv.memoize()
        self.crit_rv.memoize()

    # used to set above and beyond crit bonuses like brutal critical
    def set_crit_bonus(self, dmg_expr):
        if self.crit_rv is None:
            raise RuntimeError("Base damage not set")
        bonus_dmg_dict = self.parse_dmg_expr_(dmg_expr)
        for _, dmg in bonus_dmg_dict.items():
            self.crit_rv = self.crit_rv.add_rv(dmg)
        self.crit_rv.memoize()

    def set_resisted(self, resist):
        self.resisted = resist

    def create_die_rv_(self, num, size, reroll = None):
        base_die_rv = None
        if reroll is None:
            base_die_rv = Dice(size)
        else:
            base_die_rv = DiceReroll(size, reroll)
        total_die_rv = base_die_rv
        if num > 1:
            for x in range(num-1):
                total_die_rv = total_die_rv.add_rv(base_die_rv)
        return total_die_rv

    def parse_dmg_expr_(self, expr):
        damage = {}
        static_dmg = 0
        parts = [part.strip() for part in expr.split('+')]
        for part in parts:
            if 'r' in part:
                new_parts = part.split('r')
                reroll = int(new_parts[1])
                if 'd' in part:
                    final_parts = new_parts[0].split('d')
                    num_dice = int(final_parts[0])
                    die_size = int(final_parts[1])
                    damage[part] = self.create_die_rv_(num_dice, die_size, reroll)
                else:
                    raise RuntimeError("Can't have 'r' without 'd'")
            elif 'd' in part:
                new_parts = part.split('d')
                num_dice = int(new_parts[0])
                die_size = int(new_parts[1])
                damage[part] = self.create_die_rv_(num_dice, die_size)
            else:
                static_dmg += int(part)
        damage[self.static_dmg_key] = Constant(static_dmg)
        return damage


