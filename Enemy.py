#!/usr/bin/python3

from HitEnums import *

class Enemy(object):

    def __init__(self, ac, hit_type = HitType.NORMAL, auto_crit = False):
        self.ac = ac
        self.hit_type = hit_type
        self.auto_crit = auto_crit

    def set_ac(self, ac):
        self.ac = ac

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
        return Enemy(self.ac, self.hit_type, self.auto_crit)


