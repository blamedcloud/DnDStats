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

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum HitDice {
    D6,
    D8,
    D10,
    D12,
}

impl HitDice {
    pub fn get_max(&self) -> isize {
        match self {
            HitDice::D6 => 6,
            HitDice::D8 => 8,
            HitDice::D10 => 10,
            HitDice::D12 => 12,
        }
    }

    pub fn get_per_lvl(&self) -> isize {
        match self {
            HitDice::D6 => 4,
            HitDice::D8 => 5,
            HitDice::D10 => 6,
            HitDice::D12 => 7,
        }
    }
}
