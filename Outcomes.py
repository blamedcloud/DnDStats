#!/usr/bin/python3

from RandomVariable import *

class Outcomes(object):

    def __init__(self):
        self.outcome_chance_dict = None
        self.outcome_rv_dict = None
        # currently this does nothing
        self.condense_outliers = True
        self.cap_lb = None
        self.cap_ub = None

    def set_outcome_chances(self, d):
        self.outcome_chance_dict = d

    def set_outcome_rvs(self, d):
        self.outcome_rv_dict = d

    def set_condense(self, condense):
        self.condense_outliers = condense

    def set_cap_lb(self, lb):
        self.cap_lb = lb

    def set_cap_ub(self, ub):
        self.cap_ub = ub

    def get_outcome_chance(self, outcome):
        return self.outcome_chance_dict[outcome]

    def get_outcome_rv(self, outcome):
        return self.outcome_rv_dict[outcome]

    def describe_outcomes(self, approx = False):
        total = 0
        for outcome in self.outcome_chance_dict:
            outcome_chance = self.get_outcome_chance(outcome)
            total += outcome_chance
            if approx:
                print(outcome,":",outcome_chance,"~=",float(outcome_chance))
            else:
                print(outcome,":",outcome_chance)
        print("Total:",total)

    # f is a function that takes an outcome and a value, and
    # returns the probability that that outcome returns that value
    def outcome_pdf_(self, f, x):
        total = 0
        for outcome in self.outcome_chance_dict:
            total += self.get_outcome_chance(outcome) * f(outcome, x)
        return total

    def pdf(self, x):
        # normal pdf
        f = lambda o, x: self.get_outcome_rv(o).pdf(x)
        if self.cap_lb is not None:
            if x < self.cap_lb:
                return 0
            elif x == self.cap_lb:
                f = lambda o, x: self.get_outcome_rv(o).cdf(x)
        elif self.cap_ub is not None:
            if x > self.cap_ub:
                return 0
            elif x == self.cap_ub:
                f = lambda o, x: (1 - self.get_outcome_rv(o).cdf(x))
        return self.outcome_pdf_(f,x)

    def get_rv(self):
        lb = None
        ub = None
        for _, rv in self.outcome_rv_dict.items():
            if lb is None:
                lb = rv.get_lb()
            else:
                lb = min(lb,rv.get_lb())
            if ub is None:
                ub = rv.get_ub()
            else:
                ub = max(ub,rv.get_ub())
        if self.cap_lb is not None:
            lb = self.cap_lb
        if self.cap_ub is not None:
            ub = self.cap_ub
        rv = RandomVariable(lb,ub)
        rv.set_pdf(self.pdf)
        return rv


