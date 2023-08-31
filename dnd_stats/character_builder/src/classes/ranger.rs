use combat_core::actions::{ActionName, ActionType, CombatAction, CombatOption};
use combat_core::combat_event::CombatTiming;
use combat_core::conditions::{Condition, ConditionEffect, ConditionLifetime, ConditionName};
use combat_core::damage::{DamageDice, DamageFeature, DamageTerm, DamageType, ExtendedDamageDice, ExtendedDamageType};
use combat_core::damage::dice_expr::DiceExprTerm;
use combat_core::participant::ParticipantId;
use crate::{CBError, Character};
use crate::classes::{Class, ClassName, SubClass};
use crate::feature::{ExtraAttack, Feature};

// using one of the UA variant rangers because PHB ranger makes me want to vomit
pub struct VariantRangerClass;
impl Class for VariantRangerClass {
    fn get_class_name(&self) -> ClassName {
        ClassName::Ranger
    }

    fn get_static_features(&self, level: u8) -> Result<Vec<Box<dyn Feature>>, CBError> {
        match level {
            1 => Ok(Vec::new()), // TODO no-conc hunter's mark
            2 => Ok(Vec::new()),
            3 => Ok(self.get_subclass_features(level)),
            4 => Ok(Vec::new()),
            5 => Ok(vec!(Box::new(ExtraAttack(2)))),
            6 => Ok(Vec::new()),
            7 => Ok(self.get_subclass_features(level)),
            8 => Ok(Vec::new()),
            9 => Ok(Vec::new()),
            10 => Ok(Vec::new()), // TODO: fade away (go invis)
            11 => Ok(self.get_subclass_features(level)),
            12 => Ok(Vec::new()),
            13 => Ok(Vec::new()),
            14 => Ok(Vec::new()),
            15 => Ok(self.get_subclass_features(level)),
            16 => Ok(Vec::new()),
            17 => Ok(Vec::new()),
            18 => Ok(Vec::new()),
            19 => Ok(Vec::new()),
            20 => Ok(Vec::new()),
            _ => Err(CBError::InvalidLevel),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct HorizonWalkerRanger;
impl SubClass for HorizonWalkerRanger {
    fn get_class_name(&self) -> ClassName {
        ClassName::Ranger
    }

    fn get_static_features(&self, level: u8) -> Result<Vec<Box<dyn Feature>>, CBError> {
        match level {
            3 => Ok(vec!(Box::new(PlanarWarrior))),
            7 => Ok(Vec::new()),
            11 => Ok(Vec::new()), // TODO: situational third attack. Validation ?
            15 => Ok(Vec::new()),
            _ => Err(CBError::InvalidLevel),
        }
    }
}

pub struct PlanarWarrior; // TODO do more dmg at lvl 11
impl Feature for PlanarWarrior {
    fn apply(&self, character: &mut Character) -> Result<(), CBError> {
        let damage = DamageTerm::new(
            DiceExprTerm::Dice(1, ExtendedDamageDice::Basic(DamageDice::D8)),
            ExtendedDamageType::Basic(DamageType::Force)
        );

        let to_force = ConditionEffect::TakeDmgFeatureFrom(
            DamageFeature::DmgTypeConversion(DamageType::Force),
            ParticipantId::me()
        );
        let bonus_dmg = ConditionEffect::TakeBonusDmgFrom(damage, ParticipantId::me());

        let cond = Condition {
            effects: vec!(to_force, bonus_dmg),
            lifetimes: vec!(
                ConditionLifetime::OnHitByAtk(ParticipantId::me()),
                ConditionLifetime::UntilTime(CombatTiming::EndTurn(ParticipantId::me()))
            ),
        };
        let co = CombatOption::new(
            ActionType::BonusAction,
            CombatAction::ApplyComplexCondition(ConditionName::PlanarWarriorTarget, cond)
        );
        character.combat_actions.insert(ActionName::PlanarWarrior, co);
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use combat_core::ability_scores::Ability;

    use crate::Character;
    use crate::classes::{ChooseSubClass, ClassName};
    use crate::classes::ranger::HorizonWalkerRanger;
    use crate::equipment::{Armor, Equipment, OffHand, Weapon};
    use crate::feature::AbilityScoreIncrease;
    use crate::feature::feats::SharpShooter;
    use crate::feature::fighting_style::{FightingStyle, FightingStyles};
    use crate::tests::get_dex_based;

    #[test]
    fn lvl_20_rogue() {
        let equipment = Equipment::new(
            Armor::studded_leather(),
            Weapon::longbow(),
            OffHand::Free
        );
        let mut ranger = Character::new(String::from("lvl20ranger"), get_dex_based(), equipment);
        ranger.level_up(ClassName::Ranger, vec!()).unwrap();
        ranger.level_up(ClassName::Ranger, vec!(Box::new(FightingStyle(FightingStyles::Archery)))).unwrap();
        ranger.level_up(ClassName::Ranger, vec!(Box::new(ChooseSubClass(HorizonWalkerRanger)))).unwrap();
        ranger.level_up(ClassName::Ranger, vec!(Box::new(AbilityScoreIncrease::from(Ability::DEX)))).unwrap();
        ranger.level_up_basic().unwrap();
        ranger.level_up_basic().unwrap();
        ranger.level_up_basic().unwrap();
        ranger.level_up(ClassName::Ranger, vec!(Box::new(AbilityScoreIncrease::from(Ability::DEX)))).unwrap();
        ranger.level_up_basic().unwrap();
        ranger.level_up_basic().unwrap();
        ranger.level_up_basic().unwrap();
        ranger.level_up(ClassName::Ranger, vec!(Box::new(SharpShooter))).unwrap();
        ranger.level_up_basic().unwrap();
        ranger.level_up_basic().unwrap();
        ranger.level_up_basic().unwrap();
        ranger.level_up(ClassName::Ranger, vec!(Box::new(AbilityScoreIncrease::from(Ability::CON)))).unwrap();
        ranger.level_up_basic().unwrap();
        ranger.level_up_basic().unwrap();
        ranger.level_up(ClassName::Ranger, vec!(Box::new(AbilityScoreIncrease::from(Ability::CON)))).unwrap();
        ranger.level_up_basic().unwrap();
        assert_eq!(20, ranger.get_level());
        assert_eq!(20, ranger.ability_scores.dexterity.get_score());
        assert_eq!(20, ranger.ability_scores.constitution.get_score());

    }
}
