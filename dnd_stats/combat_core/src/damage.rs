use std::fmt::Debug;
use rand_var::RandomVariable;
use rand_var::rv_traits::NumRandVar;
use rand_var::rv_traits::prob_type::RVProb;

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum DamageDice {
    D4,
    D6,
    D8,
    D10,
    D12,
    TwoD6,
}

impl DamageDice {
    pub fn get_rv<T: RVProb>(&self) -> RandomVariable<T> {
        match self {
            DamageDice::D4 => RandomVariable::new_dice(4).unwrap(),
            DamageDice::D6 => RandomVariable::new_dice(6).unwrap(),
            DamageDice::D8 => RandomVariable::new_dice(8).unwrap(),
            DamageDice::D10 => RandomVariable::new_dice(10).unwrap(),
            DamageDice::D12 => RandomVariable::new_dice(12).unwrap(),
            DamageDice::TwoD6 => RandomVariable::new_dice(6).unwrap().multiple(2)
        }
    }

    pub fn get_rv_gwf<T: RVProb>(&self) -> RandomVariable<T> {
        match self {
            DamageDice::D4 => RandomVariable::new_dice_reroll(4, 2).unwrap(),
            DamageDice::D6 => RandomVariable::new_dice_reroll(6, 2).unwrap(),
            DamageDice::D8 => RandomVariable::new_dice_reroll(8, 2).unwrap(),
            DamageDice::D10 => RandomVariable::new_dice_reroll(10, 2).unwrap(),
            DamageDice::D12 => RandomVariable::new_dice_reroll(12, 2).unwrap(),
            DamageDice::TwoD6 => RandomVariable::new_dice_reroll(6, 2).unwrap().multiple(2)
        }
    }
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum ExtendedDamageDice {
    Basic(DamageDice),
    WeaponDice,
    SingleWeaponDie, // used for brutal critical for example
}

impl ExtendedDamageDice {
    pub fn get_single_die(dd: DamageDice) -> DamageDice {
        match dd {
            DamageDice::TwoD6 => DamageDice::D6,
            d => d,
        }
    }
}

impl From<DamageDice> for ExtendedDamageDice {
    fn from(value: DamageDice) -> Self {
        ExtendedDamageDice::Basic(value)
    }
}

#[derive(Debug, PartialEq, Copy, Clone, Eq, Hash)]
pub enum DamageFeature {
    GWF,
}

#[derive(Debug, PartialEq, Copy, Clone, Eq, Hash)]
pub enum DamageType {
    Acid,
    Bludgeoning,
    Cold,
    Fire,
    Force,
    Lightning,
    Necrotic,
    Piercing,
    Poison,
    Psychic,
    Radiant,
    Slashing,
    Thunder,
}

#[derive(Debug, PartialEq, Copy, Clone, Eq, Hash)]
pub enum ExtendedDamageType {
    Basic(DamageType),
    WeaponDamage,
}

impl From<DamageType> for ExtendedDamageType {
    fn from(value: DamageType) -> Self {
        ExtendedDamageType::Basic(value)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ExpressionTerm {
    Die(ExtendedDamageDice),
    Dice(u8, ExtendedDamageDice),
    Const(isize),
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct DamageTerm {
    pub expr: ExpressionTerm,
    pub dmg_type: ExtendedDamageType,
}

impl DamageTerm {
    pub fn new(expr: ExpressionTerm, dmg_type: ExtendedDamageType) -> Self {
        DamageTerm {
            expr,
            dmg_type,
        }
    }

    pub fn get_expr(&self) -> &ExpressionTerm {
        &self.expr
    }

    pub fn get_dmg_type(&self) -> &ExtendedDamageType {
        &self.dmg_type
    }
}

pub trait DamageRV<T: RVProb, E> : Debug {
    fn get_rv(&self) -> Result<RandomVariable<T>, E>;

    // healing is currently just negative damage
    fn get_heal_rv(&self) -> Result<RandomVariable<T>, E> {
        let rv_base = self.get_rv();
        rv_base.map(|rv| rv.opposite_rv())
    }
}
