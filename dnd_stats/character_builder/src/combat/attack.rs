use std::cmp;
use std::collections::{BTreeSet, HashSet};
use std::fmt::{Display, Formatter};
use num::{BigRational, Rational64};
use crate::ability_scores::Ability;
use crate::attributed_bonus::{BonusTerm, BonusType};
use rand_var::rv_traits::{RandVar, sequential};
use rand_var::{MapRandVar, RandomVariable};
use rand_var::rv_traits::prob_type::RVProb;
use rand_var::rv_traits::sequential::{Pair, Seq, SeqIter};
use crate::damage::{DamageDice, DamageInstance, DamageManager, DamageTerm, DamageType, ExtendedDamageDice, ExtendedDamageType};
use crate::equipment::{OffHand, Weapon, WeaponProperty, WeaponRange};
use crate::{AttributedBonus, CBError, Character};

#[derive(Debug, Copy, Clone)]
pub enum AttackHitType {
    Disadvantage,
    Normal,
    Advantage,
    SuperAdvantage,
}

impl AttackHitType {
    pub fn get_rv<T: RVProb>(&self, d20: &D20Type) -> RandomVariable<T> {
        let rv = d20.get_rv();
        match self {
            AttackHitType::Disadvantage => rv.min_two_trials(),
            AttackHitType::Normal => rv,
            AttackHitType::Advantage => rv.max_two_trials(),
            AttackHitType::SuperAdvantage => rv.max_three_trials(),
        }
    }
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum D20Type {
    D20,
    D20R1,
}

impl D20Type {
    pub fn get_rv<T: RVProb>(&self) -> RandomVariable<T> {
        match self {
            D20Type::D20 => RandomVariable::new_dice(20).unwrap(),
            D20Type::D20R1 => RandomVariable::new_dice_reroll(20, 1).unwrap()
        }
    }
}


#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum AttackResult {
    Miss,
    Hit,
    Crit,
}

// RollPair = Pair<roll, roll + bonus>
pub type RollPair = Pair<isize, isize>;

impl AttackResult {
    pub fn from(roll_pair: RollPair, ac: isize, crit_lb: isize) -> Self {
        let roll = roll_pair.0;
        let total = roll_pair.1;
        if roll == 20 {
            AttackResult::Crit
        } else if roll == 1 {
            AttackResult::Miss
        } else {
            if total >= ac {
                if roll >= crit_lb {
                    AttackResult::Crit
                } else {
                    AttackResult::Hit
                }
            } else {
                AttackResult::Miss
            }
        }
    }
}

impl Display for AttackResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut s = String::new();
        match self {
            AttackResult::Crit => s.push_str("Crit"),
            AttackResult::Hit => s.push_str("Hit"),
            AttackResult::Miss => s.push_str("Miss"),
        };
        write!(f, "{}", s)
    }
}

impl Seq for AttackResult {
    // I'm sure there's a better way to do this, but idk
    fn gen_seq(&self, other: &Self) -> SeqIter<Self> {
        let first = *cmp::min(self, other);
        let second = *cmp::max(self, other);
        let arr = [AttackResult::Miss, AttackResult::Hit, AttackResult::Crit];
        let iter= arr.iter().filter(|ar| (*ar >= &first) && (*ar <= &second));
        let items: BTreeSet<AttackResult> = iter.copied().collect();
        SeqIter { items }
    }

    fn always_convex() -> bool {
        true
    }

    fn convex_bounds(iter: SeqIter<Self>) -> Option<(Self, Self)> {
        sequential::always_convex_bounds(iter)
    }
}

pub type AccMRV<T> = MapRandVar<RollPair, T>;
pub type AccMRV64 = MapRandVar<RollPair, Rational64>;
pub type AccMRVBig = MapRandVar<RollPair, BigRational>;

pub type ArMRV<T> = MapRandVar<AttackResult, T>;
pub type ArMRV64 = MapRandVar<AttackResult, Rational64>;
pub type ArMRVBig = MapRandVar<AttackResult, BigRational>;

pub type AoMRV<T> = MapRandVar<Pair<AttackResult, isize>, T>;
pub type AoMRV64 = MapRandVar<Pair<AttackResult, isize>, Rational64>;
pub type AoMRVBig = MapRandVar<Pair<AttackResult, isize>, BigRational>;

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum NumHands {
    OneHand,
    TwoHand,
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum HandType {
    MainHand,
    OffHand,
}

#[derive(Clone)]
pub struct WeaponAttack {
    weapon: Weapon,
    num_hands: NumHands,
    hand_type: HandType,
    ability: Ability,
    hit_bonus: AttributedBonus,
    damage: DamageManager,
    crit_lb: isize,
    d20_rv: D20Type,
}

impl WeaponAttack {
    pub fn primary_weapon(character: &Character) -> Self {
        let weapon = character.get_equipment().get_primary_weapon();
        let mut hands = NumHands::OneHand;
        if character.get_equipment().get_off_hand() == &OffHand::Free {
            if weapon.has_property(WeaponProperty::TwoHanded) || weapon.is_versatile().is_some() {
                hands = NumHands::TwoHand;
            }
        }
        let mut attack = WeaponAttack::new(weapon, HandType::MainHand, hands);
        attack.cache_char_vals(character);
        attack
    }

    pub fn offhand_weapon(character: &Character) -> Option<Self> {
        character.get_equipment().get_secondary_weapon().map(|w| {
            let mut attack = WeaponAttack::new(w, HandType::OffHand, NumHands::OneHand);
            attack.cache_char_vals(character);
            attack
        })
    }

    fn get_weapon_die(weapon: &Weapon, num_hands: NumHands) -> DamageDice {
        if num_hands == NumHands::TwoHand && weapon.is_versatile().is_some() {
            *weapon.is_versatile().unwrap()
        } else {
            *weapon.get_dice()
        }
    }

    pub fn new(weapon: &Weapon, hand_type: HandType, num_hands: NumHands) -> Self {
        let mut hit_bonus = AttributedBonus::new(String::from("Hit Bonus"));
        // for now, assumes you have proficiency in the weapons you use TODO: check this
        hit_bonus.add_term(BonusTerm::new(BonusType::Proficiency));

        let mut ability = Ability::STR;
        if let WeaponRange::Ranged(_, _) = weapon.get_range() {
            ability = Ability::DEX;
        } else if weapon.has_property(WeaponProperty::Finesse) {
            // for now, assumes that if you are using a finesse
            // weapon, you use dex for it.
            // TODO: use higher ability mod
            ability = Ability::DEX;
        }
        hit_bonus.add_term(BonusTerm::new(BonusType::Modifier(ability)));

        let mut damage = DamageManager::new();
        damage.set_weapon(WeaponAttack::get_weapon_die(weapon, num_hands), *weapon.get_dmg_type());
        damage.add_base_dmg(DamageTerm::new(
            DamageInstance::Die(ExtendedDamageDice::WeaponDice),
            ExtendedDamageType::WeaponDamage,
        ));

        if hand_type == HandType::MainHand {
            damage.add_base_char_dmg(
                ExtendedDamageType::WeaponDamage,
                BonusTerm::new(BonusType::Modifier(ability)),
            );
        }

        if let Some(b) = weapon.get_magic_bonus() {
            hit_bonus.add_term(BonusTerm::new_attr(
                BonusType::Constant(b as i32),
                String::from("magic bonus"),
            ));
            damage.add_base_dmg(DamageTerm::new(
                DamageInstance::Const(b as isize),
                ExtendedDamageType::WeaponDamage,
            ));
        }

        Self {
            weapon: weapon.clone(),
            num_hands,
            hand_type,
            hit_bonus,
            ability,
            damage,
            crit_lb: 20,
            d20_rv: D20Type::D20,
        }
    }

    pub fn cache_char_vals(&mut self, character: &Character) {
        self.hit_bonus.save_value(character);
        self.damage.cache_char_dmg(character);
    }

    pub fn add_accuracy_bonus(&mut self, term: BonusTerm) {
        self.hit_bonus.add_term(term);
    }

    pub fn get_weapon(&self) -> &Weapon {
        &self.weapon
    }

    pub fn as_pam_attack(&self) -> Self {
        let mut new_attack = self.clone();
        let new_weapon = self.weapon.as_pam();
        new_attack.damage.set_weapon(*new_weapon.get_dice(), *new_weapon.get_dmg_type());
        new_attack.weapon = new_weapon;
        new_attack
    }

    pub fn get_num_hands(&self) -> &NumHands {
        &self.num_hands
    }

    pub fn get_hand_type(&self) -> &HandType {
        &self.hand_type
    }

    pub fn get_ability(&self) -> &Ability {
        &self.ability
    }

    pub fn set_crit_lb(&mut self, crit: isize) {
        if crit > 1 && crit <= 20 {
            self.crit_lb = crit;
        }
    }
    pub fn get_crit_lb(&self) -> isize {
        self.crit_lb
    }

    pub fn get_damage(&self) -> &DamageManager {
        &self.damage
    }
    pub fn get_damage_mut(&mut self) -> &mut DamageManager {
        &mut self.damage
    }

    pub fn set_d20_type(&mut self, d20type: D20Type) {
        self.d20_rv = d20type;
    }

    pub fn get_accuracy_rv<T: RVProb>(&self, hit_type: AttackHitType) -> Result<AccMRV<T>, CBError> {
        let rv = hit_type.get_rv(&self.d20_rv);
        if let None = self.hit_bonus.get_saved_value() {
            return Err(CBError::NoCache);
        }
        let hit_const = self.hit_bonus.get_saved_value().unwrap() as isize;
        Ok(rv.into_mrv().map_keys(|roll| Pair(roll, roll + hit_const)))
    }

    pub fn get_attack_result_rv<T: RVProb>(&self, hit_type: AttackHitType, target_ac: isize) -> Result<ArMRV<T>, CBError> {
        let hit_rv = self.get_accuracy_rv(hit_type)?;
        Ok(hit_rv.map_keys(|hit| AttackResult::from(hit, target_ac, self.crit_lb)))
    }

    pub fn get_attack_dmg_rv<T: RVProb>(&self, hit_type: AttackHitType, target_ac: isize, resistances: &HashSet<DamageType>) -> Result<RandomVariable<T>, CBError> {
        let attack_result_rv = self.get_attack_result_rv(hit_type, target_ac)?;
        let dmg_map = self.damage.get_attack_dmg_map(resistances)?;
        Ok(attack_result_rv.consolidate(&dmg_map)?.into())
    }

    pub fn get_attack_outcome_rv<T: RVProb>(&self, hit_type: AttackHitType, target_ac: isize, resistances: &HashSet<DamageType>) -> Result<AoMRV<T>, CBError> {
        let attack_result_rv = self.get_attack_result_rv(hit_type, target_ac)?;
        let dmg_map = self.damage.get_attack_dmg_map(resistances)?;
        Ok(attack_result_rv.projection(&dmg_map)?)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use num::{BigRational, FromPrimitive};
    use rand_var::{MRVBig, RVBig};
    use rand_var::rv_traits::NumRandVar;
    use crate::tests::get_test_fighter;
    use super::*;

    #[test]
    fn basic_atk_test() {
        let fighter = get_test_fighter();
        assert_eq!(3, fighter.get_ability_scores().strength.get_mod());
        let no_resist = HashSet::new();

        let mut attack = WeaponAttack::new(fighter.get_equipment().get_primary_weapon(), HandType::MainHand, NumHands::TwoHand);
        attack.cache_char_vals(&fighter);
        assert_eq!(&Weapon::greatsword(), attack.get_weapon());
        assert_eq!(20, attack.get_crit_lb());

        let damage = attack.get_damage();

        let two_d6: RVBig = RandomVariable::new_dice(6).unwrap().multiple(2);
        let base_dmg = two_d6.add_const(3);
        assert_eq!(base_dmg, damage.get_base_dmg(&no_resist).unwrap());
        let crit_dmg = two_d6.multiple(2).add_const(3);
        assert_eq!(crit_dmg, damage.get_crit_dmg(&no_resist).unwrap());
        let miss_dmg: RVBig = RandomVariable::new_constant(0).unwrap();
        assert_eq!(miss_dmg, damage.get_miss_dmg(&no_resist).unwrap());

        let mut dmg_map = BTreeMap::new();
        dmg_map.insert(AttackResult::Crit, crit_dmg);
        dmg_map.insert(AttackResult::Hit, base_dmg);
        dmg_map.insert(AttackResult::Miss, miss_dmg);

        let d20: MRVBig = RandomVariable::new_dice(20).unwrap().into();
        let normal_hit = d20.map_keys(|r| Pair(r, r+5));
        assert_eq!(normal_hit, attack.get_accuracy_rv(AttackHitType::Normal).unwrap());
        let target_ac = 13;
        let normal_result = normal_hit.map_keys(|hit| AttackResult::from(hit, target_ac, 20));
        let normal_dmg: RVBig = normal_result.consolidate(&dmg_map).unwrap().into();
        assert_eq!(normal_dmg, attack.get_attack_dmg_rv(AttackHitType::Normal, target_ac, &no_resist).unwrap());
    }

    #[test]
    fn validate_basic_atk() {
        let fighter = get_test_fighter();
        let attack = fighter.get_basic_attack().unwrap();
        let no_resist = HashSet::new();
        let ac = 13;
        // greatsword attack: d20 + 5 vs 13 @ 2d6 + 3
        let result_rv: ArMRVBig = attack.get_attack_result_rv(AttackHitType::Normal, ac).unwrap();
        let dmg_rv = attack.get_attack_dmg_rv(AttackHitType::Normal, ac, &no_resist).unwrap();
        assert_eq!(0, dmg_rv.lower_bound());
        assert_eq!(27, dmg_rv.upper_bound());
        assert_eq!(result_rv.pdf(AttackResult::Miss), dmg_rv.pdf(0));
        let dmg = attack.get_damage();
        let hit_ev: BigRational = dmg.get_base_dmg(&no_resist).unwrap().expected_value();
        let crit_ev: BigRational = dmg.get_crit_dmg(&no_resist).unwrap().expected_value();
        let ev = result_rv.pdf(AttackResult::Hit) * hit_ev + result_rv.pdf(AttackResult::Crit) * crit_ev;
        assert_eq!(ev, dmg_rv.expected_value());
        let attack_rv = attack.get_attack_outcome_rv(AttackHitType::Normal, ac, &no_resist).unwrap();
        assert_eq!(Pair(AttackResult::Miss, 0), attack_rv.lower_bound());
        assert_eq!(Pair(AttackResult::Crit,27), attack_rv.upper_bound());
        assert_eq!(ev, attack_rv.general_expected_value(|pair| BigRational::from_isize(pair.1).unwrap()))
    }

}
