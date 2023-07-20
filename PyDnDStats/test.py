#!/usr/bin/python3

from RandomVariable import *
from Common import *
from fractions import Fraction

if __name__ == "__main__":

    d20 = Dice(20)

    print(d20.pdf(4))
    print(d20.cdf(19))

    print(d20.expected_value())
    print(d20.expected_value(lambda x: x**2))
    print(d20.variance())

    adv = d20.max_two_trials()
    print("advantage:")
    adv.show_pdf()
    print(adv.expected_value())

    dis = d20.min_two_trials()
    print("disadvantage:")
    dis.show_pdf()
    print(dis.expected_value())

    elf_acc = d20.max_three_trials()
    print("Elven Accuracy:")
    elf_acc.show_pdf()
    print(elf_acc.expected_value())


    d6 = Dice(6)

    print("1d6")
    d6.show_pdf()

    twoD6 = d6.add_rv(d6)
    print("2d6")
    twoD6.show_pdf()

