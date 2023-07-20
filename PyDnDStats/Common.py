#!/usr/bin/python3

from RandomVariable import *
from fractions import Fraction


# rolling a die with die_size sides.
class Dice(RandomVariable):

    def __init__(self, die_size):
       super().__init__(1,die_size)
       self.set_pdf(lambda x: Fraction(1,die_size))


# rolling a die with die_size sides, rerolling once if the result is reroll_max or lower
class DiceReroll(RandomVariable):

    def __init__(self, die_size, reroll_max):
        super().__init__(1,die_size)
        def rerollPdf(x):
            if x > reroll_max:
                return Fraction(1,die_size) + Fraction(reroll_max,die_size) * Fraction(1,die_size)
            else:
                return Fraction(reroll_max,die_size) * Fraction(1,die_size)
        self.set_pdf(rerollPdf)


class Constant(RandomVariable):

    def __init__(self, value):
        super().__init__(value,value)
        self.set_pdf(lambda x: 1)


class Uniform(RandomVariable):

    def __init__(self, lb, ub):
        super().__init__(lb, ub)
        self.set_pdf(lambda x: Fraction(1, ub-lb+1))
