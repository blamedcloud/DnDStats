use std::fmt::Debug;

use character_builder::{CBError, Character};
use combat_core::CCError;
use combat_core::combat_event::CombatEvent;
use combat_core::participant::ParticipantManager;
use combat_core::strategy::{StrategyBuilder, StrategyManager};
use combat_core::strategy::strategy_impls::DoNothingBuilder;
use rand_var::rv_traits::prob_type::RVProb;
use rand_var::rv_traits::RVError;

use crate::combat_result_rv::CombatResultRV;
use crate::encounter_simulator::EncounterSimulator;
use crate::player::Player;
use crate::target_dummy::TargetDummy;

pub mod combat_result_rv;
pub mod combat_state_rv;
pub mod encounter_simulator;
pub mod monster;
pub mod player;
pub mod target_dummy;

#[derive(Debug, Clone)]
pub enum CSError {
    ActionNotHandled,
    InvalidTarget,
    InvalidAction,
    UnknownEvent(CombatEvent),
    InvalidTriggerResponse,
    RVE(RVError),
    CCE(CCError),
    CBE(CBError),
}
impl From<CBError> for CSError {
    fn from(value: CBError) -> Self {
        CSError::CBE(value)
    }
}
impl From<CCError> for CSError {
    fn from(value: CCError) -> Self {
        CSError::CCE(value)
    }
}
impl From<RVError> for CSError {
    fn from(value: RVError) -> Self {
        CSError::RVE(value)
    }
}

pub struct CombatSimulator<T: RVProb> {
    cr_rv: CombatResultRV<T>,
}

// I'm not really sure why the compiler thinks this must be static
// TODO: fix this ?
impl<T: RVProb + 'static> CombatSimulator<T> {
    pub fn do_encounter(character: Character, str_bldr: impl StrategyBuilder<T>, dummy_ac: isize, num_rounds: u8) -> Result<Self, CSError> {
        let player = Player::from(character);
        let dummy = TargetDummy::new(isize::MAX, dummy_ac);

        let mut pm = ParticipantManager::new();
        pm.add_player(Box::new(player))?;
        pm.add_enemy(Box::new(dummy))?;
        pm.compile();

        let mut sm = StrategyManager::new(&pm)?;
        sm.add_participant(str_bldr)?;
        sm.add_participant(DoNothingBuilder)?;

        let mut em: EncounterSimulator<T> = EncounterSimulator::new(&sm)?;
        em.set_do_merges(true);
        em.simulate_n_rounds(num_rounds)?;
        let cs_rv = em.get_state_rv().clone();
        Ok(Self {
            cr_rv: cs_rv.into()
        })
    }

    pub fn get_cr_rv(&self) -> &CombatResultRV<T> {
        &self.cr_rv
    }
}

#[cfg(test)]
mod tests {
    use num::Rational64;

    use character_builder::Character;
    use character_builder::classes::{ChooseSubClass, ClassName};
    use character_builder::classes::rogue::ScoutRogue;
    use character_builder::equipment::{Armor, Equipment, OffHand, Weapon};
    use character_builder::feature::fighting_style::{FightingStyle, FightingStyles};
    use combat_core::ability_scores::AbilityScores;
    use combat_core::attack::AttackHitType;
    use combat_core::participant::ParticipantId;
    use combat_core::strategy::strategy_impls::{BasicAtkStrBuilder, DualWieldStrBuilder, PairStrBuilder, SneakAttackStrBuilder};
    use rand_var::RV64;
    use rand_var::rv_traits::{NumRandVar, RandVar};

    use crate::CombatSimulator;

    pub fn get_str_based() -> AbilityScores {
        AbilityScores::new(16,12,16,8,13,10)
    }

    pub fn get_test_fighter_lvl_1() -> Character {
        let name = String::from("FighterMan");
        let ability_scores = get_str_based();
        let equipment = Equipment::new(
            Armor::chain_mail(),
            Weapon::greatsword(),
            OffHand::Free,
        );
        let mut fighter = Character::new(name, ability_scores, equipment);
        fighter.level_up(ClassName::Fighter, vec!(Box::new(FightingStyle(FightingStyles::GreatWeaponFighting)))).unwrap();
        fighter
    }

    pub fn get_dex_based() -> AbilityScores {
        AbilityScores::new(12,16,16,8,13,10)
    }

    pub fn get_test_rogue_lvl_3() -> Character {
        let name = String::from("EdgeLord");
        let ability_scores = get_dex_based();
        let equipment = Equipment::new(
            Armor::studded_leather(),
            Weapon::shortsword(),
            OffHand::Weapon(Weapon::shortsword())
        );
        let mut rogue = Character::new(name, ability_scores, equipment);
        rogue.level_up(ClassName::Rogue, vec!()).unwrap();
        rogue.level_up_basic().unwrap();
        rogue.level_up(ClassName::Rogue, vec!(Box::new(ChooseSubClass(ScoutRogue)))).unwrap();
        rogue
    }

    #[test]
    fn fighter_basic_attack_dmg() {
        let fighter = get_test_fighter_lvl_1();
        let cs = CombatSimulator::do_encounter(fighter.clone(), BasicAtkStrBuilder, 14, 1).unwrap();
        let cr_rv = cs.get_cr_rv();
        let dummy_data = cr_rv.get_pcr(0).get_participant_data().get(1).unwrap();

        let dmg_rv = cr_rv.get_dmg(ParticipantId(1));
        let atk_dmg: RV64 = fighter.get_weapon_attack().unwrap().get_attack_dmg_rv(AttackHitType::Normal, dummy_data.ac, &dummy_data.resistances).unwrap();
        assert_eq!(atk_dmg, dmg_rv);

        let cs = CombatSimulator::do_encounter(fighter.clone(), BasicAtkStrBuilder, 14, 2).unwrap();
        let cr_rv = cs.get_cr_rv();
        let dmg_rv = cr_rv.get_dmg(ParticipantId(1));
        assert_eq!(atk_dmg.multiple(2), dmg_rv);
    }

    #[test]
    fn test_sneak_attack() {
        let rogue = get_test_rogue_lvl_3();
        let rogue_str = PairStrBuilder::new(DualWieldStrBuilder, SneakAttackStrBuilder::new(false));
        let cs = CombatSimulator::do_encounter(rogue, rogue_str,14, 1).unwrap();
        let cr_rv = cs.get_cr_rv();
        {
            assert_eq!(1, cr_rv.len());

            let dmg = cr_rv.get_dmg(ParticipantId(1));
            assert_eq!(0, dmg.lower_bound());
            assert_eq!(51, dmg.upper_bound());
            assert_eq!(Rational64::new(318, 25), dmg.expected_value());
        }
    }

}
