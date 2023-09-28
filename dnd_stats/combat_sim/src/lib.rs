use std::fmt::Debug;

use character_builder::{CBError, Character};
use combat_core::CCError;
use combat_core::combat_event::CombatEvent;
use combat_core::participant::ParticipantManager;
use combat_core::strategy::{StrategyBuilder, StrategyManager};
use combat_core::strategy::basic_strategies::RemoveCondBuilder;
use rand_var::rand_var::prob_type::RVProb;
use rand_var::RVError;

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
    UnknownAction,
    UnknownEvent(CombatEvent),
    InvalidTriggerResponse,
    UncappedResource,
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

pub struct CombatSimulator<P: RVProb> {
    cr_rv: CombatResultRV<P>,
}

impl<P: RVProb> CombatSimulator<P> {
    pub fn dmg_sponge(character: Character, str_bldr: impl StrategyBuilder, dummy_ac: isize, num_rounds: u8) -> Result<Self, CSError> {
        let dummy = TargetDummy::new(isize::MAX, dummy_ac);
        CombatSimulator::vs_dummy(character, str_bldr, dummy, num_rounds)
    }

    pub fn vs_dummy(character: Character, str_bldr: impl StrategyBuilder, dummy: TargetDummy, num_rounds: u8) -> Result<Self, CSError> {
        let player = Player::from(character);
        let mut pm = ParticipantManager::new();
        pm.add_player(Box::new(player))?;
        pm.add_enemy(Box::new(dummy))?;
        pm.compile();

        let mut sm = StrategyManager::new(&pm)?;
        sm.add_participant(str_bldr)?;
        sm.add_participant(RemoveCondBuilder)?;

        let mut em: EncounterSimulator<P> = EncounterSimulator::new(&sm)?;
        em.set_do_merges(true);
        em.simulate_n_rounds(num_rounds)?;
        let cs_rv = em.get_state_rv().clone();
        Ok(Self {
            cr_rv: cs_rv.into()
        })
    }

    pub fn get_cr_rv(&self) -> &CombatResultRV<P> {
        &self.cr_rv
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::rc::Rc;
    use num::Rational64;

    use character_builder::Character;
    use character_builder::classes::{ChooseSubClass, ClassName};
    use character_builder::classes::fighter::ChampionFighter;
    use character_builder::classes::ranger::HorizonWalkerRanger;
    use character_builder::classes::rogue::ScoutRogue;
    use character_builder::classes::wizard::ConjurationWizard;
    use character_builder::equipment::{ACSource, Armor, Equipment, OffHand, Weapon};
    use character_builder::feature::AbilityScoreIncrease;
    use character_builder::feature::feats::{GreatWeaponMaster, ShieldMaster};
    use character_builder::feature::fighting_style::{FightingStyle, FightingStyles};
    use character_builder::spellcasting::cantrips::FireBoltCantrip;
    use character_builder::spellcasting::third_lvl_spells::{FireBallSpell, HasteSpell};
    use combat_core::ability_scores::{Ability, AbilityScores};
    use combat_core::D20RollType;
    use combat_core::damage::DamageType;
    use combat_core::participant::ParticipantId;
    use combat_core::strategy::action_surge_str::ActionSurgeStrBuilder;
    use combat_core::strategy::basic_atk_str::BasicAtkStrBuilder;
    use combat_core::strategy::dual_wield_str::DualWieldStrBuilder;
    use combat_core::strategy::favored_foe_str::FavoredFoeStrBldr;
    use combat_core::strategy::fireball_str::FireBallStrBuilder;
    use combat_core::strategy::firebolt_str::FireBoltStrBuilder;
    use combat_core::strategy::gwm_str::GWMStrBldr;
    use combat_core::strategy::haste_str::HasteStrBuilder;
    use combat_core::strategy::linear_str::LinearStrategyBuilder;
    use combat_core::strategy::planar_warrior_str::PlanarWarriorStrBldr;
    use combat_core::strategy::shield_master_str::ShieldMasterStrBuilder;
    use combat_core::strategy::sneak_atk_str::SneakAttackStrBuilder;
    use combat_core::strategy::StrategyBuilder;
    use rand_var::num_rand_var::NumRandVar;
    use rand_var::vec_rand_var::VRV64;
    use rand_var::rand_var::RandVar;

    use crate::CombatSimulator;
    use crate::target_dummy::TargetDummy;

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
        rogue.level_up(ClassName::Rogue, vec!(Box::new(ChooseSubClass(Rc::new(ScoutRogue))))).unwrap();
        rogue
    }

    #[test]
    fn fighter_basic_attack_dmg() {
        let fighter = get_test_fighter_lvl_1();
        let cs = CombatSimulator::dmg_sponge(fighter.clone(), BasicAtkStrBuilder, 14, 1).unwrap();
        let cr_rv = cs.get_cr_rv();
        let dummy_data = cr_rv.get_pcr(0).get_participant_data().get(1).unwrap();

        let dmg_rv = cr_rv.get_dmg(ParticipantId(1));
        let atk_dmg: VRV64 = fighter.get_weapon_attack().unwrap().get_attack_dmg_rv(D20RollType::Normal, dummy_data.ac, &dummy_data.resistances).unwrap();
        assert_eq!(atk_dmg, dmg_rv);

        let cs = CombatSimulator::dmg_sponge(fighter.clone(), BasicAtkStrBuilder, 14, 2).unwrap();
        let cr_rv = cs.get_cr_rv();
        let dmg_rv = cr_rv.get_dmg(ParticipantId(1));
        assert_eq!(atk_dmg.multiple(2), dmg_rv);
    }

    #[test]
    fn test_sneak_attack() {
        let rogue = get_test_rogue_lvl_3();
        let mut str_vec: Vec<Box<dyn StrategyBuilder>> = Vec::new();
        str_vec.push(Box::new(DualWieldStrBuilder));
        str_vec.push(Box::new(SneakAttackStrBuilder::new(false)));
        let rogue_str = LinearStrategyBuilder::from(str_vec);
        let cs = CombatSimulator::dmg_sponge(rogue, rogue_str, 14, 1).unwrap();
        let cr_rv = cs.get_cr_rv();
        {
            assert_eq!(1, cr_rv.len());

            let dmg = cr_rv.get_dmg(ParticipantId(1));
            assert_eq!(0, dmg.lower_bound());
            assert_eq!(51, dmg.upper_bound());
            assert_eq!(Rational64::new(318, 25), dmg.expected_value());
        }
    }

    #[test]
    fn test_shield_master() {
        let name = String::from("ShieldHero");
        let ability_scores = get_str_based();
        let equipment = Equipment::new(
            Armor::chain_mail(),
            Weapon::longsword(),
            OffHand::Shield(ACSource::shield()),
        );
        let mut fighter = Character::new(name, ability_scores, equipment);
        fighter.level_up(ClassName::Fighter, vec!(Box::new(FightingStyle(FightingStyles::Dueling)))).unwrap();
        fighter.level_up_basic().unwrap();
        fighter.level_up(ClassName::Fighter, vec!(Box::new(ChooseSubClass(Rc::new(ChampionFighter))))).unwrap();
        fighter.level_up(ClassName::Fighter, vec!(Box::new(ShieldMaster))).unwrap();

        let cs = CombatSimulator::dmg_sponge(fighter.clone(), ShieldMasterStrBuilder, 14, 1).unwrap();
        let cr_rv = cs.get_cr_rv();
        {
            assert_eq!(1, cr_rv.len());

            let dmg = cr_rv.get_dmg(ParticipantId(1));
            assert_eq!(0, dmg.lower_bound());
            assert_eq!(21, dmg.upper_bound());
            // we're getting to the point that I'm not sure how to verify this is
            // correct, so I'll just take the code's word for it I guess.
            assert_eq!(Rational64::new(624639, 80000), dmg.expected_value());
        }
    }

    #[test]
    fn planar_warrior_test() {
        let name = String::from("WorldHopper");
        let ability_scores = get_dex_based();
        let equipment = Equipment::new(
            Armor::studded_leather(),
            Weapon::longbow(),
            OffHand::Free,
        );
        let mut ranger = Character::new(name, ability_scores, equipment);
        ranger.level_up(ClassName::Ranger, vec!()).unwrap();
        ranger.level_up(ClassName::Ranger, vec!(Box::new(FightingStyle(FightingStyles::Archery)))).unwrap();
        ranger.level_up(ClassName::Ranger, vec!(Box::new(ChooseSubClass(Rc::new(HorizonWalkerRanger))))).unwrap();
        ranger.level_up(ClassName::Ranger, vec!(Box::new(AbilityScoreIncrease::from(Ability::DEX)))).unwrap();
        ranger.level_up_basic().unwrap();

        let mut player_str = LinearStrategyBuilder::new();
        player_str.add_str_bldr(Box::new(PlanarWarriorStrBldr));
        player_str.add_str_bldr(Box::new(BasicAtkStrBuilder));

        let mut resistances = HashSet::new();
        resistances.insert(DamageType::Slashing);
        resistances.insert(DamageType::Piercing);
        resistances.insert(DamageType::Bludgeoning);

        let angery_dummy = TargetDummy::resistant(isize::MAX, 15, resistances);

        let cs = CombatSimulator::vs_dummy(ranger.clone(), player_str, angery_dummy, 1).unwrap();
        let cr_rv = cs.get_cr_rv();
        {
            assert_eq!(1, cr_rv.len());

            let dmg = cr_rv.get_dmg(ParticipantId(1));
            assert_eq!(0, dmg.lower_bound());
            // (2*(2D8) + 4) + (2*(1D8) + 4)/2 = 46
            assert_eq!(46, dmg.upper_bound());
            assert_eq!(Rational64::new(4827, 320), dmg.expected_value());
        }
    }

    #[test]
    fn great_weapon_master_test() {
        let name = String::from("Cloud");
        let ability_scores = get_str_based();
        let equipment = Equipment::new(
            Armor::chain_mail(),
            Weapon::greatsword(),
            OffHand::Free,
        );
        let mut fighter = Character::new(name, ability_scores, equipment);
        fighter.level_up(ClassName::Fighter, vec!(Box::new(FightingStyle(FightingStyles::GreatWeaponFighting)))).unwrap();
        fighter.level_up_basic().unwrap();
        fighter.level_up(ClassName::Fighter, vec!(Box::new(ChooseSubClass(Rc::new(ChampionFighter))))).unwrap();
        fighter.level_up(ClassName::Fighter, vec!(Box::new(GreatWeaponMaster))).unwrap();
        fighter.level_up_basic().unwrap();
        fighter.level_up(ClassName::Fighter, vec!(Box::new(AbilityScoreIncrease::from(Ability::STR)))).unwrap();

        let cs = CombatSimulator::dmg_sponge(fighter.clone(), GWMStrBldr::new(true), 15, 1).unwrap();
        let cr_rv = cs.get_cr_rv();
        {
            assert_eq!(1, cr_rv.len());

            let dmg = cr_rv.get_dmg(ParticipantId(1));
            assert_eq!(0, dmg.lower_bound());
            // 3*(2*(2D6) + 14) = 114
            assert_eq!(114, dmg.upper_bound());
            assert_eq!(Rational64::new(21389, 1000), dmg.expected_value());
        }
    }

    #[test]
    fn action_surge_choose_no_gmw_test() {
        let name = String::from("Cloud");
        let ability_scores = get_str_based();
        let equipment = Equipment::new(
            Armor::chain_mail(),
            Weapon::greatsword(),
            OffHand::Free,
        );
        let mut fighter = Character::new(name, ability_scores, equipment);
        fighter.level_up(ClassName::Fighter, vec!(Box::new(FightingStyle(FightingStyles::GreatWeaponFighting)))).unwrap();
        fighter.level_up_basic().unwrap();
        fighter.level_up(ClassName::Fighter, vec!(Box::new(ChooseSubClass(Rc::new(ChampionFighter))))).unwrap();
        fighter.level_up(ClassName::Fighter, vec!(Box::new(GreatWeaponMaster))).unwrap();

        let mut fighter_str = LinearStrategyBuilder::new();
        fighter_str.add_str_bldr(Box::new(ActionSurgeStrBuilder));
        fighter_str.add_str_bldr(Box::new(GWMStrBldr::new(false)));

        let cs = CombatSimulator::dmg_sponge(fighter.clone(), fighter_str, 15, 1).unwrap();
        let cr_rv = cs.get_cr_rv();
        {
            assert_eq!(1, cr_rv.len());

            let dmg = cr_rv.get_dmg(ParticipantId(1));
            assert_eq!(0, dmg.lower_bound());
            // 3*(2*(2D6) + 3) = 81
            assert_eq!(81, dmg.upper_bound());
            assert_eq!(Rational64::new(3869, 250), dmg.expected_value());
        }
    }

    #[test]
    fn wizard_fire_bolt_test() {
        let name = String::from("bell");
        let ability_scores = AbilityScores::new(10,14,14,16,12,8);
        let equipment = Equipment::new(
            Armor::no_armor(),
            Weapon::quarterstaff(),
            OffHand::Free,
        );
        let mut wizard = Character::new(name, ability_scores, equipment);
        wizard.level_up(ClassName::Wizard, vec!()).unwrap();
        wizard.level_up(ClassName::Wizard, vec!(Box::new(ChooseSubClass(Rc::new(ConjurationWizard))))).unwrap();
        wizard.level_up(ClassName::Wizard, vec!(Box::new(FireBoltCantrip(Ability::INT)))).unwrap();
        let wizard_str = FireBoltStrBuilder;

        let cs = CombatSimulator::dmg_sponge(wizard.clone(), wizard_str, 13, 1).unwrap();
        let cr_rv = cs.get_cr_rv();
        {
            assert_eq!(1, cr_rv.len());

            let dmg = cr_rv.get_dmg(ParticipantId(1));
            assert_eq!(0, dmg.lower_bound());
            assert_eq!(20, dmg.upper_bound());
            assert_eq!(Rational64::new(77, 20), dmg.expected_value());
        }
    }

    #[test]
    fn wizard_fireball_test() {
        let name = String::from("elaine");
        let ability_scores = AbilityScores::new(10,14,14,16,12,8);
        let equipment = Equipment::new(
            Armor::no_armor(),
            Weapon::quarterstaff(),
            OffHand::Free,
        );
        let mut wizard = Character::new(name, ability_scores, equipment);
        wizard.level_up(ClassName::Wizard, vec!()).unwrap();
        wizard.level_up(ClassName::Wizard, vec!(Box::new(ChooseSubClass(Rc::new(ConjurationWizard))))).unwrap();
        wizard.level_up_basic().unwrap();
        wizard.level_up(ClassName::Wizard, vec!(Box::new(AbilityScoreIncrease::from(Ability::INT)))).unwrap();
        wizard.level_up(ClassName::Wizard, vec!(Box::new(FireBallSpell(Ability::INT)))).unwrap();
        let wizard_str = FireBallStrBuilder;

        let cs = CombatSimulator::dmg_sponge(wizard.clone(), wizard_str, 15, 1).unwrap();
        let cr_rv = cs.get_cr_rv();
        {
            assert_eq!(1, cr_rv.len());

            let dmg = cr_rv.get_dmg(ParticipantId(1));
            assert_eq!(4, dmg.lower_bound());
            assert_eq!(48, dmg.upper_bound());
            assert_eq!(Rational64::new(1727, 80), dmg.expected_value());
        }
    }

    #[test]
    fn bonus_action_spell_rule() {
        let name = String::from("Speedy");
        let ability_scores = AbilityScores::new(12,16,16,8,13,10);
        let equipment = Equipment::new(
            Armor::studded_leather(),
            Weapon::longbow(),
            OffHand::Free,
        );
        let mut ranger = Character::new(name, ability_scores, equipment);
        ranger.level_up(ClassName::Ranger, vec!()).unwrap();
        ranger.level_up(ClassName::Ranger, vec!(Box::new(FightingStyle(FightingStyles::Archery)))).unwrap();
        ranger.level_up(ClassName::Ranger, vec!(Box::new(ChooseSubClass(Rc::new(HorizonWalkerRanger))))).unwrap();
        ranger.level_up(ClassName::Ranger, vec!(Box::new(AbilityScoreIncrease::from(Ability::DEX)))).unwrap();
        ranger.level_up_basic().unwrap();
        ranger.level_up_basic().unwrap();
        ranger.level_up_basic().unwrap();
        ranger.level_up(ClassName::Ranger, vec!(Box::new(AbilityScoreIncrease::from(Ability::DEX)))).unwrap();
        ranger.level_up(ClassName::Ranger, vec!(Box::new(HasteSpell))).unwrap();

        let mut player_str = LinearStrategyBuilder::new();
        player_str.add_str_bldr(Box::new(HasteStrBuilder));
        player_str.add_str_bldr(Box::new(FavoredFoeStrBldr));
        player_str.add_str_bldr(Box::new(BasicAtkStrBuilder));
    }
}
