#!/usr/bin/python3

import math

def summation(lb,ub,f):
    result = 0
    for i in range(lb,ub+1):
        result += f(i)
    return result

def convolution(f1,f2,lb,ub):
    def conv_x(x):
        return summation(lb,ub,lambda y: f1(x-y)*f2(y))
    return conv_x


class RandomVariable(object):

    def __init__(self, lower_bound, upper_bound):
        self.lower_bound = lower_bound
        self.upper_bound = upper_bound
        self.pdf_ = None
        self.cdf_ = None

    def set_pdf(self, f):
        self.pdf_ = f

    def set_cdf(self, F):
        self.cdf_ = F

    def get_lb(self):
        return self.lower_bound

    def get_ub(self):
        return self.upper_bound

    def is_constant(self):
        return self.lower_bound == self.upper_bound

    def copy(self):
        copy = RandomVariable(self.lower_bound, self.upper_bound)
        pdf_dict = self.get_pdf_dict()
        def newPdf(x):
            return pdf_dict[x]
        copy.set_pdf(newPdf)
        return copy

    def memoize(self):
        pdf_dict = self.get_pdf_dict()
        def newPdf(x):
            return pdf_dict[x]
        self.set_pdf(newPdf)

    def get_pdf_dict(self):
        pdf_dict = {}
        for x in range(self.lower_bound,self.upper_bound+1):
            pdf_dict[x] = self.pdf(x)
        return pdf_dict

    def pdf(self, x):
        if x < self.lower_bound:
            return 0
        elif x > self.upper_bound:
            return 0
        else:
            if self.pdf_ is not None:
                return self.pdf_(x)
            else:
                raise RuntimeError("PDF not defined")

    def show_pdf(self, approx = False):
        for x in range(self.lower_bound, self.upper_bound+1):
            if approx:
                print(x,":",self.pdf(x),"~=",float(self.pdf(x)))
            else:
                print(x,":",self.pdf(x))

    def describe(self, approx = True):
        print("PDF:")
        self.show_pdf(approx)
        print("CDF(" + str(self.get_ub()) + ") =", self.cdf(self.get_ub()))
        self.show_stats(approx)

    def show_stats(self, approx = True):
        print("Bounds: (" + str(self.lower_bound) + ", " + str(self.upper_bound) + ")")
        mu = self.expected_value()
        if approx:
            print("mu =",mu,"~=",float(mu))
        else:
            print("mu =",mu)
        var = self.variance()
        if approx:
            print("var=",var,"~=",float(var))
        else:
            print("var=",var)
        print("std.dev~=",math.sqrt(var))


    def cdf(self, x):
        if self.cdf_ is None:
            if self.pdf_ is not None:
                return summation(self.lower_bound,x,self.pdf)
            else:
                raise RuntimeError("CDF and PDF not defined")
        else:
            return self.cdf_(x)

    def expected_value(self, f = lambda x: x):
        return summation(self.lower_bound,self.upper_bound,lambda x: f(x) * self.pdf(x))

    def variance(self):
        return self.expected_value(lambda x: x**2) - self.expected_value()**2

    def max_two_trials(self):
        maxVar = RandomVariable(self.lower_bound, self.upper_bound)
        def maxPdf(x):
            return 2 * self.pdf(x) * self.cdf(x-1) + self.pdf(x)**2
        maxVar.set_pdf(maxPdf)
        return maxVar

    def min_two_trials(self):
        minVar = RandomVariable(self.lower_bound, self.upper_bound)
        # f_min2(x) = 2f(x) - f_max2(x)
        def minPdf(x):
            return 2 * self.pdf(x) - (2 * self.pdf(x) * self.cdf(x-1) + self.pdf(x)**2)
        minVar.set_pdf(minPdf)
        return minVar

    def max_three_trials(self):
        maxVar = RandomVariable(self.lower_bound, self.upper_bound)
        def max3Pdf(x):
            return 3 * self.pdf(x) * self.cdf(x-1)**2 + 3 * (self.pdf(x) ** 2) * self.cdf(x-1) + self.pdf(x) ** 3
        maxVar.set_pdf(max3Pdf)
        return maxVar

    def add_rv(self, other):
        sumVar = RandomVariable(self.lower_bound+other.lower_bound, self.upper_bound+other.upper_bound)
        true_lb = min(self.lower_bound, other.lower_bound)
        true_ub = max(self.upper_bound, other.upper_bound)
        sumVar.set_pdf(convolution(self.pdf, other.pdf, true_lb, true_ub))
        return sumVar

    def half_round_down(self):
        lb = math.floor(self.lower_bound/2)
        ub = math.floor(self.upper_bound/2)
        def halfPdf(x):
            return self.pdf(2*x) + self.pdf(2*x+1)
        halfVar = RandomVariable(lb,ub)
        halfVar.set_pdf(halfPdf)
        return halfVar


