use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Display;
use rv_partition::RVPartition;
use crate::rand_var::prob_type::{Prob, Reciprocal};
use crate::rand_var::sequential::SeqIter;
use crate::RVError;

pub mod sequential;
pub mod prob_type;
pub mod rv_partition;

pub trait RandVar<K, P>
where
    K: Ord + Clone,
    P: Prob,
{
    fn build<F: Fn(K) -> P>(seq_iter: SeqIter<K>, f: F) -> Result<Self, RVError> where Self: Sized;
    fn lower_bound(&self) -> K;
    fn upper_bound(&self) -> K;
    unsafe fn raw_pdf(&self, k: &K) -> P;
    fn get_keys(&self) -> SeqIter<K>;
    fn len(&self) -> usize;

    fn pdf_ref(&self, k: &K) -> P {
        if (&self.lower_bound() <= k) && (k <= &self.upper_bound()) {
            unsafe {
                self.raw_pdf(k)
            }
        } else {
            P::zero()
        }
    }
    fn pdf(&self, k: K) -> P {
        self.pdf_ref(&k)
    }

    fn cdf_ref(&self, k: &K) -> P {
        if &self.upper_bound() <= k {
            P::one()
        } else if &self.lower_bound() <= k {
            self.get_keys().take_while(|x| x <= k).map(|x| self.pdf(x)).sum()
        } else {
            P::zero()
        }
    }
    fn cdf(&self, k: K) -> P {
        self.cdf_ref(&k)
    }

    fn cdf_exclusive_ref(&self, k: &K) -> P {
        if &self.upper_bound() < k {
            P::one()
        } else if &self.lower_bound() < k {
            self.get_keys().take_while(|x| x < k).map(|x| self.pdf(x)).sum()
        } else {
            P::zero()
        }
    }
    fn cdf_exclusive(&self, k: K) -> P {
        self.cdf_exclusive_ref(&k)
    }

    fn general_expected_value<F>(&self, f: F) -> P
    where
        F: Fn(K) -> P
    {
        let mut result = P::zero();
        for k in self.get_keys() {
            result = result + self.pdf_ref(&k) * f(k);
        }
        result
    }

    fn cap_lb(&self, lb: K) -> Result<Self, RVError>
    where
        Self: Sized + Clone,
    {
        self.cab_lb_force(lb, false)
    }

    fn cab_lb_force(&self, lb: K, force: bool) -> Result<Self, RVError>
    where
        Self: Sized + Clone,
    {
        if lb > self.lower_bound() {
            let seq_iter = SeqIter { items: self.get_keys().filter(|k| k >= &lb).collect() };
            RandVar::build(seq_iter, |k| {
                if k == lb {
                    self.cdf_ref(&lb)
                } else {
                    self.pdf(k)
                }
            })
        } else {
            if lb < self.lower_bound() && force {
                let mut new_keys = self.get_keys().items;
                new_keys.insert(lb.clone());
                RandVar::build(SeqIter { items: new_keys }, |k| {
                    if k == lb {
                        P::zero()
                    } else {
                        self.pdf(k)
                    }
                })
            } else {
                Ok(self.clone())
            }
        }
    }

    fn cap_ub(&self, ub: K) -> Result<Self, RVError>
    where
        Self: Sized + Clone,
    {
        self.cap_ub_force(ub, false)
    }

    fn cap_ub_force(&self, ub: K, force: bool) -> Result<Self, RVError>
    where
        Self: Sized + Clone,
    {
        if ub < self.upper_bound() {
            let seq_iter = SeqIter { items: self.get_keys().filter(|k| k <= &ub).collect() };
            RandVar::build(seq_iter, |k| {
                if k < ub {
                    self.pdf(k)
                } else { // k == ub
                    P::one() - self.cdf_exclusive_ref(&ub)
                }
            })
        } else {
            if ub > self.upper_bound() && force {
                let mut new_keys = self.get_keys().items;
                new_keys.insert(ub.clone());
                RandVar::build(SeqIter { items: new_keys }, |k| {
                    if k == ub {
                        P::zero()
                    } else {
                        self.pdf(k)
                    }
                })
            } else {
                Ok(self.clone())
            }
        }
    }

    fn pdf_on<F>(&self, pred: F) -> P
    where
        F: Fn(&K) -> bool,
    {
        self.get_keys().filter(pred).map(|k| self.pdf(k)).sum()
    }

    fn get_partition<F>(&self, pred: F) -> RVPartition<K, P, Self>
    where
        Self: Sized,
        F: Fn(&K) -> bool,
        P: Reciprocal,
    {
        let filter_k: BTreeSet<K> = self.get_keys().filter(pred).collect();
        if filter_k.len() > 0 {
            let filter_si = SeqIter { items: filter_k };
            let prob_on: P = filter_si.clone().map(|k| self.pdf(k)).sum();
            let prob_on_recip = prob_on.clone().reciprocal().unwrap();
            let slice_rv = RandVar::build(filter_si, |k| self.pdf(k) * prob_on_recip.clone()).unwrap();
            RVPartition::new(prob_on, slice_rv)
        } else {
            RVPartition::empty()
        }
    }

    fn partitions<F, K2>(&self, part: F) -> BTreeMap<K2, RVPartition<K, P, Self>>
    where
        Self: Sized,
        K2: Ord + Clone,
        F: Fn(&K) -> K2,
        P: Reciprocal + PartialOrd<P>,
    {
        let part_map: BTreeMap<K, K2> = self.get_keys().map(|k| {
            let k2 = part(&k);
            (k, k2)
        }).collect();
        let k2_vals: BTreeSet<K2> = part_map.iter().map(|(_, k2)| k2.clone()).collect();
        let mut result = BTreeMap::new();
        for k2 in k2_vals.into_iter() {
            let k_set: BTreeSet<K> = part_map.iter()
                .filter(|(_, v)| *v == &k2)
                .map(|(k, _)| k.clone())
                .collect();
            let partition = self.get_partition(|k| k_set.contains(k));
            if partition.prob > P::zero() {
                result.insert(k2, partition);
            }
        }
        result
    }

    fn reroll_once_on<F>(&self, pred: F) -> Self
    where
        F: Fn(&K) -> bool,
        Self: Sized,
    {
        let reroll_chance = self.pdf_on(&pred);
        let reroll_pdf = |k| {
            if pred(&k) {
                reroll_chance.clone() * self.pdf(k)
            } else {
                (P::one() + reroll_chance.clone()) * self.pdf(k)
            }
        };
        // .unwrap() is fine here, because if self is a valid RV, then this also will be.
        RandVar::build(self.get_keys(), reroll_pdf).unwrap()
    }

    fn max_two_trials(&self) -> Self
    where
        Self: Sized
    {
        let max_pdf = |k| {
            (P::one()+ P::one()) * self.pdf_ref(&k) * self.cdf_exclusive_ref(&k) + num::pow(self.pdf_ref(&k), 2)
        };
        // .unwrap() is fine here, because if self is a valid RV, then this also will be.
        RandVar::build(self.get_keys(), max_pdf).unwrap()
    }

    fn min_two_trials(&self) -> Self
    where
        Self: Sized
    {
        let min_pdf = |k| {
            let max_pdf = (P::one()+ P::one()) * self.pdf_ref(&k) * self.cdf_exclusive_ref(&k) + num::pow(self.pdf_ref(&k), 2);
            (P::one()+ P::one()) * self.pdf_ref(&k) - max_pdf
        };
        // .unwrap() is fine here, because if self is a valid RV, then this also will be.
        RandVar::build(self.get_keys(), min_pdf).unwrap()
    }

    fn max_three_trials(&self) -> Self
    where
        Self: Sized
    {
        let max_pdf = |k| {
            let x = (P::one() + P::one() + P::one()) * self.pdf_ref(&k) * num::pow(self.cdf_exclusive_ref(&k), 2);
            let y = (P::one() + P::one() + P::one()) * num::pow(self.pdf_ref(&k), 2) * self.cdf_exclusive_ref(&k);
            x + y + num::pow(self.pdf_ref(&k), 3)
        };
        // .unwrap() is fine here, because if self is a valid RV, then this also will be.
        RandVar::build(self.get_keys(), max_pdf).unwrap()
    }

    fn print_distributions(&self)
    where
        K: Display,
        P: Display,
    {
        println!("K:\tpdf\t(cdf)");
        for k in self.get_keys() {
            println!("{}: {}\t\t({})", &k, self.pdf_ref(&k), self.cdf_ref(&k));
        }
    }

    fn print_pdf<F, P2>(&self, f: &F)
    where
        K: Display,
        P: Display,
        F: Fn(P) -> P2,
        P2: Display,
    {
        println!("K\tpdf\t~pdf");
        for k in self.get_keys() {
            println!("{}: {}\t\t~{}", &k, self.pdf_ref(&k), f(self.pdf_ref(&k)));
        }
    }
}

#[cfg(test)]
mod tests {
    use num::{One, Rational64, Zero};

    use crate::num_rand_var::NumRandVar;
    use crate::vec_rand_var::{VecRandVar, VRV64};

    use super::*;

    #[test]
    fn test_reroll() {
        let rv1: VRV64 = VecRandVar::new_dice_reroll(10, 2).unwrap();
        let rv2: VRV64 = VecRandVar::new_dice(10).unwrap().reroll_once_on(|p| *p <= 2);
        assert_eq!(rv1, rv2);

        let rv3: VRV64 = VecRandVar::new_dice_reroll(20, 1).unwrap();
        let rv4: VRV64 = VecRandVar::new_dice(20).unwrap().reroll_once_on(|p| *p <= 1);
        assert_eq!(rv3, rv4);
    }

    #[test]
    fn test_d20_adv() {
        let rv: VRV64 = VecRandVar::new_dice(20).unwrap().max_two_trials();
        assert_eq!(1, rv.lower_bound());
        assert_eq!(20, rv.upper_bound());
        assert_eq!(Rational64::zero(), rv.pdf(0));
        assert_eq!(Rational64::zero(), rv.pdf(21));

        let mut total = Rational64::zero();
        for x in 1..=20 {
            assert_eq!(Rational64::new(2*x-1,400), rv.pdf(x as isize));
            total += rv.pdf(x as isize);
            assert_eq!(total, rv.cdf(x as isize));
        }
        assert_eq!(Rational64::one(), total);
        assert_eq!(Rational64::new(553,40), rv.expected_value());
    }

    #[test]
    fn test_d20_disadv() {
        let rv: VRV64 = VecRandVar::new_dice(20).unwrap().min_two_trials();
        assert_eq!(1, rv.lower_bound());
        assert_eq!(20, rv.upper_bound());
        assert_eq!(Rational64::zero(), rv.pdf(0));
        assert_eq!(Rational64::zero(), rv.pdf(21));

        let mut total = Rational64::zero();
        for x in 1..=20 {
            assert_eq!(Rational64::new(2*(21-x)-1,400), rv.pdf(x as isize));
            total += rv.pdf(x as isize);
            assert_eq!(total, rv.cdf(x as isize));
        }
        assert_eq!(Rational64::one(), total);
        assert_eq!(Rational64::new(287,40), rv.expected_value());
    }

    #[test]
    fn test_d20_super_adv() {
        let rv: VRV64 = VecRandVar::new_dice(20).unwrap().max_three_trials();
        assert_eq!(1, rv.lower_bound());
        assert_eq!(20, rv.upper_bound());
        assert_eq!(Rational64::zero(), rv.pdf(0));
        assert_eq!(Rational64::zero(), rv.pdf(21));
        let mut total = Rational64::zero();
        for x in 1..=20 {
            assert_eq!(Rational64::new(3*x*x-3*x+1,8000), rv.pdf(x as isize));
            total += rv.pdf(x as isize);
            assert_eq!(total, rv.cdf(x as isize));
        }
        assert_eq!(Rational64::one(), total);
        assert_eq!(Rational64::new(1239,80), rv.expected_value());
    }

    #[test]
    fn test_cmp_rv() {
        let d6: VRV64 = VecRandVar::new_dice(6).unwrap();
        assert_eq!(Rational64::new(5,12), d6.prob_gt(&d6));
        assert_eq!(Rational64::new(7,12), d6.prob_ge(&d6));
        assert_eq!(Rational64::new(1,6), d6.prob_eq(&d6));
        assert_eq!(Rational64::new(7,12), d6.prob_le(&d6));
        assert_eq!(Rational64::new(5,12), d6.prob_gt(&d6));

        let d20: VRV64 = VecRandVar::new_dice(20).unwrap();
        let d20_adv = d20.max_two_trials();
        assert_eq!(Rational64::one(), d20_adv.prob_ge(&d20) + d20_adv.prob_lt(&d20));
        assert_eq!(Rational64::one(), d20_adv.prob_gt(&d20) + d20_adv.prob_lt(&d20) + d20_adv.prob_eq(&d20));
    }
}
