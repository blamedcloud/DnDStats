use rand_var::RandomVariable;
use rand_var::rv_traits::prob_type::RVProb;
use rand_var::rv_traits::RandVar;

#[derive(Debug, Ord, PartialOrd, PartialEq, Eq, Clone, Copy)]
pub enum Health {
    Healthy,
    Bloodied,
    ZeroHP,
    Dead,
}

impl Health {
    pub fn classify_bounds<T: RVProb>(dmg: &RandomVariable<T>, max_hp: isize, dead_at_zero: bool) -> (Health, Health) {
        let bloodied = Health::calc_bloodied(max_hp);
        let lb = dmg.lower_bound();
        let ub = dmg.upper_bound();
        let lb_h = Health::classify_hp(&lb, bloodied, max_hp, dead_at_zero);
        let ub_h = Health::classify_hp(&ub, bloodied, max_hp, dead_at_zero);
        (lb_h, ub_h)
    }

    pub fn calc_bloodied(max_hp: isize) -> isize {
        if max_hp % 2 == 0 {
            max_hp / 2
        } else {
            (max_hp / 2) + 1
        }
    }

    pub fn classify_hp(dmg: &isize, bloody_hp: isize, max_hp: isize, dead_at_zero: bool) -> Health {
        if dmg < &bloody_hp {
            Health::Healthy
        } else if dmg < &max_hp {
            Health::Bloodied
        } else {
            if dead_at_zero {
                Health::Dead
            } else {
                Health::ZeroHP
            }
        }
    }
}
