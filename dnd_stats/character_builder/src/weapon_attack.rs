use std::collections::HashSet;

use combat_core::ability_scores::Ability;
use combat_core::attack::{AccMRV, AoMRV, ArMRV, AtkDmgMap, Attack, AttackHitType, AttackResult, D20Type};
use combat_core::CCError;
use combat_core::damage::{DamageDice, DamageTerm, DamageType, ExpressionTerm, ExtendedDamageDice, ExtendedDamageType};
use rand_var::RandomVariable;
use rand_var::rv_traits::prob_type::RVProb;
use rand_var::rv_traits::sequential::Pair;

use crate::{CBError, Character};
use crate::attributed_bonus::{AttributedBonus, BonusTerm, BonusType};
use crate::damage_manager::DamageManager;
use crate::equipment::{OffHand, Weapon, WeaponProperty, WeaponRange};

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


#[derive(Debug, Clone)]
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
            ExpressionTerm::Die(ExtendedDamageDice::WeaponDice),
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
                ExpressionTerm::Const(b as isize),
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

    pub fn set_d20_type(&mut self, d20type: D20Type) {
        self.d20_rv = d20type;
    }

    pub fn get_d20_type(&self) -> &D20Type {
        &self.d20_rv
    }

    fn get_crit_lb(&self) -> isize {
        self.crit_lb
    }

    pub fn set_crit_lb(&mut self, crit: isize) {
        if crit > 1 && crit <= 20 {
            self.crit_lb = crit;
        }
    }

    pub fn get_hit_bonus(&self) -> &AttributedBonus {
        &self.hit_bonus
    }

    pub fn get_damage(&self) -> &DamageManager {
        &self.damage
    }
    pub fn get_damage_mut(&mut self) -> &mut DamageManager {
        &mut self.damage
    }

    pub fn get_dmg_map<T: RVProb>(&self, resistances: &HashSet<DamageType>) -> Result<AtkDmgMap<T>, CBError> {
        Ok(self.damage.get_attack_dmg_map(resistances)?)
    }

    pub fn get_accuracy_rv<T: RVProb>(&self, hit_type: AttackHitType) -> Result<AccMRV<T>, CBError> {
        let rv = hit_type.get_rv(&self.d20_rv);
        if let None = self.hit_bonus.get_saved_value() {
            return Err(CBError::NoCache.into());
        }
        let hit_const = self.hit_bonus.get_saved_value().unwrap() as isize;
        Ok(rv.into_mrv().map_keys(|roll| Pair(roll, roll + hit_const)))
    }

    pub fn get_attack_result_rv<T: RVProb>(&self, hit_type: AttackHitType, target_ac: isize) -> Result<ArMRV<T>, CBError> {
        let hit_rv = self.get_accuracy_rv(hit_type)?;
        Ok(hit_rv.map_keys(|hit| AttackResult::from(hit, target_ac, self.get_crit_lb())))
    }

    pub fn get_attack_dmg_rv<T: RVProb>(&self, hit_type: AttackHitType, target_ac: isize, resistances: &HashSet<DamageType>) -> Result<RandomVariable<T>, CBError> {
        let attack_result_rv = self.get_attack_result_rv(hit_type, target_ac)?;
        let dmg_map = self.get_dmg_map(resistances)?;
        Ok(attack_result_rv.consolidate(&dmg_map.into_ar_map())?.into())
    }

    pub fn get_attack_outcome_rv<T: RVProb>(&self, hit_type: AttackHitType, target_ac: isize, resistances: &HashSet<DamageType>) -> Result<AoMRV<T>, CBError> {
        let attack_result_rv = self.get_attack_result_rv(hit_type, target_ac)?;
        let dmg_map = self.get_dmg_map(resistances)?;
        Ok(attack_result_rv.projection(&dmg_map.into_ar_map())?)
    }
}

impl<T: RVProb> Attack<T> for WeaponAttack {
    fn get_miss_dmg(&self, resistances: &HashSet<DamageType>, bonus_dmg: Vec<DamageTerm>) -> Result<RandomVariable<T>, CCError> {
        Ok(self.damage.get_miss_dmg(resistances, bonus_dmg)?)
    }

    fn get_hit_dmg(&self, resistances: &HashSet<DamageType>, bonus_dmg: Vec<DamageTerm>) -> Result<RandomVariable<T>, CCError> {
        Ok(self.damage.get_base_dmg(resistances, bonus_dmg)?)
    }

    fn get_crit_dmg(&self, resistances: &HashSet<DamageType>, bonus_dmg: Vec<DamageTerm>) -> Result<RandomVariable<T>, CCError> {
        Ok(self.damage.get_crit_dmg(resistances, bonus_dmg)?)
    }

    fn get_acc_rv(&self, hit_type: AttackHitType) -> Result<AccMRV<T>, CCError> {
        let rv = hit_type.get_rv(self.get_d20_type());
        if let None = self.get_hit_bonus().get_saved_value() {
            return Err(CBError::NoCache.into());
        }
        let hit_const = self.get_hit_bonus().get_saved_value().unwrap() as isize;
        Ok(rv.into_mrv().map_keys(|roll| Pair(roll, roll + hit_const)))
    }

    fn get_dmg_map(&self, resistances: &HashSet<DamageType>) -> Result<AtkDmgMap<T>, CCError> {
        Ok(self.get_damage().get_attack_dmg_map(resistances)?)
    }

    fn get_crit_lb(&self) -> isize {
        self.get_crit_lb()
    }


}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, HashSet};

    use num::{BigRational, FromPrimitive};

    use combat_core::attack::{ArMRVBig, AttackHitType, AttackResult};
    use rand_var::{MRVBig, RandomVariable, RVBig};
    use rand_var::rv_traits::{NumRandVar, RandVar};
    use rand_var::rv_traits::sequential::Pair;

    use crate::equipment::Weapon;
    use crate::tests::get_test_fighter;
    use crate::weapon_attack::{HandType, NumHands, WeaponAttack};

    #[test]
    fn weapon_atk_test() {
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
        assert_eq!(base_dmg, damage.get_base_dmg(&no_resist, vec!()).unwrap());
        let crit_dmg = two_d6.multiple(2).add_const(3);
        assert_eq!(crit_dmg, damage.get_crit_dmg(&no_resist, vec!()).unwrap());
        let miss_dmg: RVBig = RandomVariable::new_constant(0).unwrap();
        assert_eq!(miss_dmg, damage.get_miss_dmg(&no_resist, vec!()).unwrap());

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
    fn validate_weapon_atk() {
        let fighter = get_test_fighter();
        let attack = fighter.get_weapon_attack().unwrap();
        let no_resist = HashSet::new();
        let ac = 13;
        // greatsword attack: d20 + 5 vs 13 @ 2d6 + 3
        let result_rv: ArMRVBig = attack.get_attack_result_rv(AttackHitType::Normal, ac).unwrap();
        let dmg_rv = attack.get_attack_dmg_rv(AttackHitType::Normal, ac, &no_resist).unwrap();
        assert_eq!(0, dmg_rv.lower_bound());
        assert_eq!(27, dmg_rv.upper_bound());
        assert_eq!(result_rv.pdf(AttackResult::Miss), dmg_rv.pdf(0));
        let dmg = attack.get_damage();
        let hit_ev: BigRational = dmg.get_base_dmg(&no_resist, vec!()).unwrap().expected_value();
        let crit_ev: BigRational = dmg.get_crit_dmg(&no_resist, vec!()).unwrap().expected_value();
        let ev = result_rv.pdf(AttackResult::Hit) * hit_ev + result_rv.pdf(AttackResult::Crit) * crit_ev;
        assert_eq!(ev, dmg_rv.expected_value());
        let attack_rv = attack.get_attack_outcome_rv(AttackHitType::Normal, ac, &no_resist).unwrap();
        assert_eq!(Pair(AttackResult::Miss, 0), attack_rv.lower_bound());
        assert_eq!(Pair(AttackResult::Crit,27), attack_rv.upper_bound());
        assert_eq!(ev, attack_rv.general_expected_value(|pair| BigRational::from_isize(pair.1).unwrap()))
    }

}
