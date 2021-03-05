#!/usr/bin/python3

from HitEnums import *
from Common import Constant
from RandomVariable import RandomVariable

class Enemy(object):

    def __init__(self, ac, hit_type = HitType.NORMAL, auto_crit = False):
        self.hit_type = hit_type
        self.auto_crit = auto_crit
        if isinstance(ac, int):
            self.ac = Constant(ac)
        elif isinstance(ac, RandomVariable):
            self.ac = ac
        else:
            raise RuntimeError("ac should be an int or an RV")

    def set_ac(self, ac):
        if isinstance(ac, int):
            self.ac = Constant(ac)
        elif isinstance(ac, RandomVariable):
            self.ac = ac
        else:
            raise RuntimeError("ac should be an int or an RV")

    def get_ac(self):
        return self.ac

    def set_hit_type(self, hit_type):
        self.hit_type = hit_type

    def apply_hit_type(self, new_hit_type):
        if self.hit_type == HitType.NORMAL:
            self.hit_type = new_hit_type
        elif new_hit_type == HitType.NORMAL:
            pass
        elif self.hit_type == self.new_hit_type:
            pass
        else:
            self.hit_type = HitType.NORMAL

    def get_hit_type(self):
        return self.hit_type

    def set_auto_crit(self, auto_crit):
        self.auto_crit = auto_crit

    def get_auto_crit(self):
        return self.auto_crit

    def copy(self):
        return Enemy(self.ac.copy(), self.hit_type, self.auto_crit)


