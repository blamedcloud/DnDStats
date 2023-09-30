use combat_core::actions::{ActionName, ActionType, CombatAction, CombatOption};
use combat_core::damage::{DamageDice, ExtendedDamageDice};
use combat_core::damage::dice_expr::{DiceExpr, DiceExprTerm};
use combat_core::resources::{RefreshTiming, Resource, ResourceActionType, ResourceName};
use combat_core::resources::resource_amounts::{RefreshBy, ResourceCap};

use crate::{CBError, Character};
use crate::attributed_bonus::{BonusTerm, BonusType};
use crate::classes::{Class, ClassName, SubClass};
use crate::char_damage::CharDiceExpr;
use crate::feature::{ExtraAttack, Feature};

pub struct FighterClass;
impl Class for FighterClass {
    fn get_class_name(&self) -> ClassName {
        ClassName::Fighter
    }

    fn get_static_features(&self, level: u8) -> Result<Vec<Box<dyn Feature>>, CBError> {
        match level {
            1 => Ok(vec!(Box::new(SecondWind))),
            2 => Ok(vec!(Box::new(ActionSurge(1)))),
            3 => Ok(self.get_subclass_features(level)),
            4 => Ok(Vec::new()),
            5 => Ok(vec!(Box::new(ExtraAttack(2)))),
            6 => Ok(Vec::new()),
            7 => Ok(self.get_subclass_features(level)),
            8 => Ok(Vec::new()),
            9 => Ok(vec!(Box::new(Indomitable(1)))),
            10 => Ok(self.get_subclass_features(level)),
            11 => Ok(vec!(Box::new(ExtraAttack(3)))),
            12 => Ok(Vec::new()),
            13 => Ok(vec!(Box::new(Indomitable(2)))),
            14 => Ok(Vec::new()),
            15 => Ok(self.get_subclass_features(level)),
            16 => Ok(Vec::new()),
            17 => Ok(vec!(Box::new(Indomitable(3)), Box::new(ActionSurge(2)))),
            18 => Ok(self.get_subclass_features(level)),
            19 => Ok(Vec::new()),
            20 => Ok(vec!(Box::new(ExtraAttack(4)))),
            _ => Err(CBError::InvalidLevel)
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct ChampionFighter;
impl SubClass for ChampionFighter {
    fn get_class_name(&self) -> ClassName {
        ClassName::Fighter
    }

    fn get_static_features(&self, level: u8) -> Result<Vec<Box<dyn Feature>>, CBError> {
        match level {
            3 => Ok(vec!(Box::new(ImprovedCritical(19)))),
            7 => Ok(Vec::new()),
            10 => Ok(Vec::new()),
            15 => Ok(vec!(Box::new(ImprovedCritical(18)))),
            18 => Ok(Vec::new()),
            _ => Err(CBError::InvalidLevel),
        }
    }
}

pub struct SecondWind;
impl Feature for SecondWind {
    fn apply(&self, character: &mut Character) -> Result<(), CBError> {
        let mut heal = CharDiceExpr::new();
        heal.add_term(DiceExprTerm::Die(ExtendedDamageDice::Basic(DamageDice::D10)));
        heal.add_char_term(BonusTerm::new(BonusType::ClassLevel(ClassName::Fighter)));
        character.combat_actions.insert(ActionName::SecondWind, CombatOption::new(ActionType::BonusAction, CombatAction::SelfHeal(heal)));

        let mut res = Resource::from(ResourceCap::Hard(1));
        res.add_refresh(RefreshTiming::ShortRest, RefreshBy::ToFull);
        res.add_refresh(RefreshTiming::LongRest, RefreshBy::ToFull);
        character.resource_manager.add_perm(ResourceName::AN(ActionName::SecondWind), res);

        Ok(())
    }
}

pub struct ActionSurge(pub usize);
impl Feature for ActionSurge {
    fn apply(&self, character: &mut Character) -> Result<(), CBError> {
        character.combat_actions.insert(
            ActionName::ActionSurge,
            CombatOption::new(
                ActionType::FreeAction,
                CombatAction::GainResource(ResourceName::RAT(ResourceActionType::Action), 1)
            )
        );

        let mut res = Resource::from(ResourceCap::Hard(self.0));
        res.add_refresh(RefreshTiming::ShortRest, RefreshBy::ToFull);
        res.add_refresh(RefreshTiming::LongRest, RefreshBy::ToFull);
        character.resource_manager.add_perm(ResourceName::AN(ActionName::ActionSurge), res);

        Ok(())
    }
}

pub struct Indomitable(pub usize);
impl Feature for Indomitable {
    fn apply(&self, character: &mut Character) -> Result<(), CBError> {
        // TODO: change action type to OnSave ?
        character.combat_actions.insert(ActionName::Indomitable, CombatOption::new(ActionType::FreeAction, CombatAction::ByName));

        let mut res = Resource::from(ResourceCap::Hard(self.0));
        res.add_refresh(RefreshTiming::LongRest, RefreshBy::ToFull);
        character.resource_manager.add_perm(ResourceName::AN(ActionName::ActionSurge), res);

        Ok(())
    }
}

pub struct ImprovedCritical(pub isize);
impl Feature for ImprovedCritical {
    fn apply(&self, character: &mut Character) -> Result<(), CBError> {
        for (_, co) in character.combat_actions.iter_mut() {
            if let CombatAction::Attack(wa) = &mut co.action {
                wa.set_crit_lb(self.0);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;
    use combat_core::ability_scores::Ability;

    use crate::Character;
    use crate::classes::{ChooseSubClass, ClassName};
    use crate::classes::fighter::ChampionFighter;
    use crate::equipment::{Armor, Equipment, OffHand, Weapon};
    use crate::feature::AbilityScoreIncrease;
    use crate::feature::feats::{GreatWeaponMaster, Resilient};
    use crate::feature::fighting_style::{FightingStyle, FightingStyles};
    use crate::tests::get_str_based;

    #[test]
    fn lvl_20_fighter() {
        let equipment = Equipment::new(
            Armor::chain_mail(),
            Weapon::greatsword(),
            OffHand::Free
        );
        let mut fighter = Character::new(String::from("lvl20fighter"), get_str_based(), equipment);
        fighter.level_up(ClassName::Fighter, vec!(Box::new(FightingStyle(FightingStyles::GreatWeaponFighting)))).unwrap();
        fighter.level_up_basic().unwrap();
        fighter.level_up(ClassName::Fighter, vec!(Box::new(ChooseSubClass(Rc::new(ChampionFighter))))).unwrap();
        fighter.level_up(ClassName::Fighter, vec!(Box::new(GreatWeaponMaster))).unwrap();
        fighter.level_up_basic().unwrap();
        fighter.level_up(ClassName::Fighter, vec!(Box::new(AbilityScoreIncrease::from(Ability::STR)))).unwrap();
        fighter.level_up_basic().unwrap();
        fighter.level_up(ClassName::Fighter, vec!(Box::new(AbilityScoreIncrease::from(Ability::STR)))).unwrap();
        fighter.level_up_basic().unwrap();
        fighter.level_up_basic().unwrap();
        fighter.level_up_basic().unwrap();
        fighter.level_up(ClassName::Fighter, vec!(Box::new(AbilityScoreIncrease::from(Ability::CON)))).unwrap();
        fighter.level_up_basic().unwrap();
        fighter.level_up(ClassName::Fighter, vec!(Box::new(AbilityScoreIncrease::from(Ability::CON)))).unwrap();
        fighter.level_up_basic().unwrap();
        fighter.level_up(ClassName::Fighter, vec!(Box::new(Resilient(Ability::WIS)))).unwrap();
        fighter.level_up_basic().unwrap();
        fighter.level_up_basic().unwrap();
        fighter.level_up(ClassName::Fighter, vec!(Box::new(AbilityScoreIncrease::from(Ability::WIS)))).unwrap();
        fighter.level_up_basic().unwrap();
        assert_eq!(20, fighter.get_level());
        assert_eq!(20, fighter.ability_scores.strength.get_score());
        assert_eq!(20, fighter.ability_scores.constitution.get_score());
        assert_eq!(16, fighter.ability_scores.wisdom.get_score());
    }
}
