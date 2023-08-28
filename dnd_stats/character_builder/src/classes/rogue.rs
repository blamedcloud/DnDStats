use combat_core::ability_scores::Ability;
use combat_core::damage::{DamageDice, DamageTerm, ExpressionTerm, ExtendedDamageDice, ExtendedDamageType};
use combat_core::resources::{RefreshBy, RefreshTiming, Resource, ResourceCap, ResourceName};
use combat_core::triggers::{TriggerAction, TriggerName, TriggerType};

use crate::{CBError, Character};
use crate::classes::{Class, ClassName, SubClass};
use crate::feature::{Feature, SaveProficiencies};

pub struct RogueClass;
impl RogueClass {
    pub fn sneak_attack(&self, level: u8) -> Box<SneakAttack> {
        Box::new(SneakAttack((level + 1) / 2))
    }
}
impl Class for RogueClass {
    fn get_class_name(&self) -> ClassName {
        ClassName::Rogue
    }

    fn get_static_features(&self, level: u8) -> Result<Vec<Box<dyn Feature>>, CBError> {
        match level {
            1 => Ok(vec!(self.sneak_attack(level))),
            2 => Ok(Vec::new()), // TODO: impl cunning action if movement or hiding happen
            3 => {
                let mut v = self.get_subclass_features(level);
                v.push(self.sneak_attack(level));
                Ok(v)
            },
            4 => Ok(Vec::new()),
            5 => Ok(vec!(self.sneak_attack(level))), // TODO: uncanny dodge: when reactions get impl'd
            6 => Ok(Vec::new()),
            7 => Ok(vec!(self.sneak_attack(level))), // TODO: impl Evasion: How?
            8 => Ok(Vec::new()),
            9 => {
                let mut v = self.get_subclass_features(level);
                v.push(self.sneak_attack(level));
                Ok(v)
            },
            10 => Ok(Vec::new()),
            11 => Ok(vec!(self.sneak_attack(level))), // TODO: reliable talent when ability checks are relevant: grappling?
            12 => Ok(Vec::new()),
            13 => {
                let mut v = self.get_subclass_features(level);
                v.push(self.sneak_attack(level));
                Ok(v)
            },
            14 => Ok(Vec::new()),
            15 => Ok(vec!(self.sneak_attack(level), Box::new(SaveProficiencies::from(Ability::WIS)))), // Slippery Mind
            16 => Ok(Vec::new()),
            17 => {
                let mut v = self.get_subclass_features(level);
                v.push(self.sneak_attack(level));
                Ok(v)
            },
            18 => Ok(Vec::new()), // TODO: impl Elusive: as permanent condition ?
            19 => Ok(vec!(self.sneak_attack(level))),
            20 => Ok(Vec::new()), // TODO: impl stroke of luck: OnMiss ActionType ?
            _ => Err(CBError::InvalidLevel)
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct ScoutRogue;
impl SubClass for ScoutRogue {
    fn get_class_name(&self) -> ClassName {
        ClassName::Rogue
    }

    // TODO: impl some features
    fn get_static_features(&self, level: u8) -> Result<Vec<Box<dyn Feature>>, CBError> {
        match level {
            3 => Ok(Vec::new()),
            9 => Ok(Vec::new()),
            13 => Ok(Vec::new()),
            17 => Ok(Vec::new()),
            _ => Err(CBError::InvalidLevel),
        }
    }
}

pub struct SneakAttack(pub u8); // TODO: check conditions for this: How?
impl Feature for SneakAttack {
    fn apply(&self, character: &mut Character) -> Result<(), CBError> {
        let damage = DamageTerm::new(
            ExpressionTerm::Dice(self.0, ExtendedDamageDice::Basic(DamageDice::D6)),
            ExtendedDamageType::WeaponDamage
        );

        let response = (TriggerAction::AddAttackDamage(damage), ResourceName::TN(TriggerName::SneakAttack)).into();
        character.trigger_manager.add_trigger(TriggerType::SuccessfulAttack, TriggerName::SneakAttack);
        character.trigger_manager.set_response(TriggerName::SneakAttack, response);

        let mut res = Resource::from(ResourceCap::Hard(1));
        // refreshing at the end of turns instead of beginning in order to
        // take advantage of transpositions.
        res.add_refresh(RefreshTiming::EndMyTurn, RefreshBy::ToFull);
        res.add_refresh(RefreshTiming::EndOtherTurn, RefreshBy::ToFull);
        character.resource_manager.add_perm(ResourceName::TN(TriggerName::SneakAttack), res);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use combat_core::ability_scores::Ability;
    use combat_core::damage::{DamageDice, ExpressionTerm, ExtendedDamageDice, ExtendedDamageType};
    use combat_core::triggers::{TriggerAction, TriggerName};

    use crate::Character;
    use crate::classes::{ChooseSubClass, ClassName};
    use crate::classes::rogue::ScoutRogue;
    use crate::equipment::{Armor, Equipment, OffHand, Weapon};
    use crate::feature::AbilityScoreIncrease;
    use crate::tests::get_dex_based;

    #[test]
    fn lvl_20_rogue() {
        let equipment = Equipment::new(
            Armor::studded_leather(),
            Weapon::dagger(),
            OffHand::Weapon(Weapon::dagger())
        );
        let mut rogue = Character::new(String::from("lvl20rogue"), get_dex_based(), equipment);
        rogue.level_up(ClassName::Rogue, vec!()).unwrap();
        rogue.level_up_basic().unwrap();
        rogue.level_up(ClassName::Rogue, vec!(Box::new(ChooseSubClass(ScoutRogue)))).unwrap();
        rogue.level_up(ClassName::Rogue, vec!(Box::new(AbilityScoreIncrease::from(Ability::DEX)))).unwrap();
        rogue.level_up_basic().unwrap();
        rogue.level_up_basic().unwrap();
        rogue.level_up_basic().unwrap();
        rogue.level_up(ClassName::Rogue, vec!(Box::new(AbilityScoreIncrease::from(Ability::DEX)))).unwrap();
        rogue.level_up_basic().unwrap();
        rogue.level_up(ClassName::Rogue, vec!(Box::new(AbilityScoreIncrease::from(Ability::CON)))).unwrap();
        rogue.level_up_basic().unwrap();
        rogue.level_up(ClassName::Rogue, vec!(Box::new(AbilityScoreIncrease::from(Ability::CON)))).unwrap();
        rogue.level_up_basic().unwrap();
        rogue.level_up_basic().unwrap();
        rogue.level_up_basic().unwrap();
        rogue.level_up(ClassName::Rogue, vec!(Box::new(AbilityScoreIncrease::from(Ability::WIS)))).unwrap();
        rogue.level_up_basic().unwrap();
        rogue.level_up_basic().unwrap();
        rogue.level_up(ClassName::Rogue, vec!(Box::new(AbilityScoreIncrease::from(Ability::WIS)))).unwrap();
        rogue.level_up_basic().unwrap();
        assert_eq!(20, rogue.get_level());
        assert_eq!(20, rogue.ability_scores.dexterity.get_score());
        assert_eq!(20, rogue.ability_scores.constitution.get_score());
        assert_eq!(17, rogue.ability_scores.wisdom.get_score());
        let snr = &rogue.trigger_manager.get_response(TriggerName::SneakAttack).unwrap();
        if let TriggerAction::AddAttackDamage(dt) = snr.action {
            assert_eq!(ExtendedDamageType::WeaponDamage, *dt.get_dmg_type());
            assert_eq!(ExpressionTerm::Dice(10, ExtendedDamageDice::Basic(DamageDice::D6)), *dt.get_expr())
        } else {
            panic!("Wrong sneak attack action");
        }
    }
}
