#!/usr/bin/python3

from RandomVariable import *
from Common import *
from fractions import Fraction

if __name__ == "__main__":

    d6 = Dice(6)
    constant = Constant(3)
    damage = d6.add_rv(constant)
    damage = damage.add_rv(d6)
    damage.show_pdf()
    print(damage.expected_value())

    print("---")

    d6r2 = DiceReroll(6,2)
    damage_gwf = d6r2.add_rv(d6r2)
    damage_gwf = damage_gwf.add_rv(constant)
    damage_gwf.show_pdf()
    print(damage_gwf.expected_value())
