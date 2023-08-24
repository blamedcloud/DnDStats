use std::collections::BTreeMap;
use num::{BigRational, Rational64};
use character_builder::{CBError, Character};
use character_builder::combat::{ActionName, ActionType, CombatAction};
use character_builder::combat::attack::{ArMRV, AttackHitType, WeaponAttack};
use character_builder::resources::{ResourceManager, ResourceName};
use rand_var::{MapRandVar, RandomVariable};
use rand_var::rv_traits::prob_type::RVProb;
use crate::combat_log::{CombatEvent, CombatStateRV, CombatTiming, ProbCombatState};
use crate::participant::{Participant, ParticipantId, Team, TeamMember};
use crate::strategy::{StrategicOption, Strategy, Target};

pub mod combat_log;
pub mod participant;
pub mod strategy;
pub mod target_dummy;

#[derive(Debug)]
pub enum CSError {
    ActionNotHandled,
    InvalidTarget,
    CBE(CBError)
}
impl From<CBError> for CSError {
    fn from(value: CBError) -> Self {
        CSError::CBE(value)
    }
}

pub enum HandledAction<T: RVProb> {
    InPlace(ProbCombatState<T>),
    Children(Vec<ProbCombatState<T>>)
}

type ResultHA<T> = Result<HandledAction<T>, CSError>;
type ResultCSE = Result<(), CSError>;

pub struct EncounterManager<T: RVProb> {
    participants: Vec<TeamMember>, // in order of initiative
    strategies: Vec<Box<dyn Strategy>>,
    round_num: u8,
    cs_rv: CombatStateRV<T>,
}

pub type EM64 = EncounterManager<Rational64>;
pub type EMBig = EncounterManager<BigRational>;

impl<T: RVProb> EncounterManager<T> {
    pub fn new() -> Self {
        Self {
            participants: Vec::new(),
            strategies: Vec::new(),
            round_num: 0,
            cs_rv: CombatStateRV::new(),
        }
    }

    pub fn add_player(&mut self, character: Character, str: Box<dyn Strategy>) {
        let rm = character.get_resource_manager().clone();
        self.add_participant(TeamMember::new(Team::Players, Box::new(character)), rm, str);
    }

    pub fn add_participant(&mut self, tm: TeamMember, rm: ResourceManager, str: Box<dyn Strategy>) {
        self.cs_rv.add_participant(rm, tm.participant.get_max_hp());
        self.participants.push(tm);
        self.strategies.push(str);
    }

    pub fn simulate_n_rounds(&mut self, n: u8) -> ResultCSE {
        for _ in 0..n {
            self.round_num += 1;
            self.register_timing(CombatTiming::BeginRound(self.round_num.into()));
            self.simulate_round()?;
            self.register_timing(CombatTiming::EndRound(self.round_num.into()));
        }
        Ok(())
    }

    pub fn get_state_rv(&self) -> &CombatStateRV<T> {
        &self.cs_rv
    }

    fn register_timing(&mut self, ct: CombatTiming) {
        let event = CombatEvent::Timing(ct);
        for pcs in self.cs_rv.get_states_mut() {
            pcs.push(event);

            for i in 0..self.participants.len() {
                let pid = ParticipantId(i);
                let rt = ct.get_refresh_timing(pid);
                pcs.get_rm_mut(pid).handle_timing(rt);
            }
        }
    }

    fn simulate_round(&mut self) -> ResultCSE {
        for i in 0..self.participants.len() {
            let pid = ParticipantId(i);
            self.register_timing(CombatTiming::BeginTurn(pid));
            self.simulate_turn(pid)?;
            self.register_timing(CombatTiming::EndTurn(pid));
        }
        Ok(())
    }

    fn simulate_turn(&mut self, pid: ParticipantId) -> ResultCSE {
        let mut finished_pcs = Vec::new();
        for pcs in self.cs_rv.get_states() {
            let new_pcs = self.finish_turn(pcs.clone(), pid)?;
            finished_pcs.extend(new_pcs.into_iter());
        }
        self.cs_rv = finished_pcs.into();
        Ok(())
    }

    fn get_strategy(&self, pid: ParticipantId) -> &Box<dyn Strategy> {
        &self.strategies.get(pid.0).unwrap()
    }

    fn finish_turn(&self, pcs: ProbCombatState<T>, pid: ParticipantId) -> Result<Vec<ProbCombatState<T>>, CSError> {
        let strategy = self.get_strategy(pid);
        if let Some(so) = strategy.get_action(pcs.get_state(), &self.participants, pid) {
            if self.possible_action(&pcs, pid, so) {
                let handled_action = self.handle_action(pcs, pid, so)?;
                match handled_action {
                    HandledAction::InPlace(p) => self.finish_turn(p, pid),
                    HandledAction::Children(children) => {
                        let mut finished_pcs = Vec::new();
                        for pcs in children.into_iter() {
                            let new_pcs = self.finish_turn(pcs, pid)?;
                            finished_pcs.extend(new_pcs.into_iter());
                        }
                        Ok(finished_pcs)
                    }
                }
            } else {
                // strategy gave me an invalid StrategicOption
                Ok(vec!(pcs))
            }
        } else {
            // strategy end-turn on purpose.
            Ok(vec!(pcs))
        }
    }

    fn get_participant(&self, pid: ParticipantId) -> &Box<dyn Participant> {
        &self.participants.get(pid.0).unwrap().participant
    }

    fn possible_action(&self, pcs: &ProbCombatState<T>, pid: ParticipantId, so: StrategicOption) -> bool {
        let an = so.action_name;
        let participant = self.get_participant(pid);
        let has_action = participant.get_action_manager().contains_key(&an);
        let mut valid_target = true;
        let mut has_resources = true;
        if has_action {
            let co = participant.get_action_manager().get(&an).unwrap();
            if co.req_target && so.target.is_none() {
                valid_target = false;
            }
        }
        if has_action && valid_target {
            let rm = pcs.get_rm(pid);
            if rm.has_resource(ResourceName::AN(an)) && rm.get_current(ResourceName::AN(an)) == 0 {
                has_resources = false;
            } else {
                let co = participant.get_action_manager().get(&an).unwrap();
                let at = co.action_type;
                if rm.has_resource(ResourceName::AT(at)) && rm.get_current(ResourceName::AT(at)) == 0 {
                    has_resources = false;
                }
            }
        }
        has_action && valid_target && has_resources
    }

    fn handle_action(&self, mut pcs: ProbCombatState<T>, pid: ParticipantId, so: StrategicOption) -> ResultHA<T> {
        let an = so.action_name;
        let participant = self.get_participant(pid);
        let co = participant.get_action_manager().get(&an).unwrap();
        let at = co.action_type;
        pcs.spend_resources(pid, an, at);

        pcs.push(CombatEvent::AN(an));

        match &co.action {
            CombatAction::Attack(wa) => {
                if let Target::Participant(target_pid) = so.target.unwrap() {
                    pcs.push(CombatEvent::Attack(pid, target_pid));
                    Ok(HandledAction::Children(self.handle_attack(pcs, wa, target_pid)?))
                } else {
                    Err(CSError::InvalidTarget)
                }
            },
            CombatAction::AdditionalAttacks(aa) => {
                pcs.get_rm_mut(pid).gain(ResourceName::AT(ActionType::SingleAttack), *aa as usize);
                Ok(HandledAction::InPlace(pcs))
            },
            CombatAction::ByName => {
                match an {
                    ActionName::ActionSurge => {
                        pcs.get_rm_mut(pid).gain(ResourceName::AT(ActionType::Action), 1);
                        Ok(HandledAction::InPlace(pcs))
                    },
                    _ => Err(CSError::ActionNotHandled),
                }
            }
        }
    }

    fn handle_attack(&self, pcs: ProbCombatState<T>, wa: &WeaponAttack, target_pid: ParticipantId) -> Result<Vec<ProbCombatState<T>>, CBError> {
        let target = self.get_participant(target_pid);
        let outcome_rv: ArMRV<T> = wa.get_attack_result_rv(AttackHitType::Normal, target.get_ac())?;
        let ce_rv: MapRandVar<CombatEvent, T> = outcome_rv.map_keys(|ar| ar.into());
        let ar_dmg_map = wa.get_dmg_map(target.get_resistances())?;
        let ce_dmg_map: BTreeMap<CombatEvent, RandomVariable<T>> = ar_dmg_map.into_iter().map(|(k, v)| (k.into(), v)).collect();
        Ok(pcs.split_dmg(ce_rv, ce_dmg_map, target_pid))
    }
}

#[cfg(test)]
mod tests {
    use num::{One, Rational64};
    use character_builder::ability_scores::AbilityScores;
    use character_builder::Character;
    use character_builder::classes::ClassName;
    use character_builder::combat::attack::{AttackHitType, AttackResult};
    use character_builder::combat::{ActionName, AttackType};
    use character_builder::equipment::{Armor, Equipment, OffHand, Weapon};
    use character_builder::feature::fighting_style::{FightingStyle, FightingStyles};
    use character_builder::resources::ResourceManager;
    use rand_var::RV64;
    use rand_var::rv_traits::NumRandVar;
    use crate::{EM64, EncounterManager};
    use crate::combat_log::{CombatEvent, CombatTiming, RoundId};
    use crate::participant::{Participant, ParticipantId, Team, TeamMember};
    use crate::strategy::{BasicAttacks, DoNothing};
    use crate::target_dummy::TargetDummy;

    pub fn get_str_based() -> AbilityScores {
        AbilityScores::new(16,12,16,8,13,10)
    }

    pub fn get_test_fighter_lvl_0() -> Character {
        let name = String::from("FighterMan");
        let ability_scores = get_str_based();
        let equipment = Equipment::new(
            Armor::chain_mail(),
            Weapon::greatsword(),
            OffHand::Free,
        );
        Character::new(name, ability_scores, equipment)
    }

    #[test]
    fn fighter_vs_dummy_do_nothing() {
        let mut fighter = get_test_fighter_lvl_0();
        fighter.level_up(ClassName::Fighter, vec!(Box::new(FightingStyle(FightingStyles::GreatWeaponFighting)))).unwrap();
        let dummy = TargetDummy::new(isize::MAX, 14);

        let mut em: EM64 = EncounterManager::new();
        em.add_player(fighter, Box::new(DoNothing));
        em.add_participant(TeamMember::new(Team::Enemies, Box::new(dummy)), ResourceManager::new(),Box::new(DoNothing));
        em.simulate_n_rounds(1).unwrap();

        let cs_rv = em.get_state_rv();
        assert_eq!(1, cs_rv.len());

        let pcs = cs_rv.get_pcs(0);
        assert_eq!(Rational64::one(), *pcs.get_prob());

        let logs = pcs.get_state().get_logs();
        assert!(!logs.has_parent());

        let events = logs.get_local_events();
        assert_eq!(6, events.len());
        let all_events = logs.get_all_events();
        assert_eq!(all_events, events.clone());

        let expected_events = vec!(
            CombatEvent::Timing(CombatTiming::BeginRound(RoundId(1))),
            CombatEvent::Timing(CombatTiming::BeginTurn(ParticipantId(0))),
            CombatEvent::Timing(CombatTiming::EndTurn(ParticipantId(0))),
            CombatEvent::Timing(CombatTiming::BeginTurn(ParticipantId(1))),
            CombatEvent::Timing(CombatTiming::EndTurn(ParticipantId(1))),
            CombatEvent::Timing(CombatTiming::EndRound(RoundId(1)))
        );
        assert_eq!(all_events, expected_events);
    }

    #[test]
    fn fighter_vs_dummy_basic_attack() {
        let mut fighter = get_test_fighter_lvl_0();
        fighter.level_up(ClassName::Fighter, vec!(Box::new(FightingStyle(FightingStyles::GreatWeaponFighting)))).unwrap();
        let dummy = TargetDummy::new(isize::MAX, 14);

        let mut em: EM64 = EncounterManager::new();
        em.add_player(fighter.clone(), Box::new(BasicAttacks));
        em.add_participant(TeamMember::new(Team::Enemies, Box::new(dummy.clone())), ResourceManager::new(),Box::new(DoNothing));
        em.simulate_n_rounds(1).unwrap();

        let cs_rv = em.get_state_rv();
        assert_eq!(3, cs_rv.len());

        let index_rv = cs_rv.get_index_rv();
        let ar_rv: RV64 = fighter.get_basic_attack().unwrap()
            .get_attack_result_rv(AttackHitType::Normal, dummy.get_ac()).unwrap()
            .map_keys(|ar| {
                match ar {
                    AttackResult::Miss => 0,
                    AttackResult::Hit => 1,
                    AttackResult::Crit => 2
                }
            }).into_rv();
        assert_eq!(ar_rv, index_rv);

        let hit_pcs = cs_rv.get_pcs(1);
        let hit_logs = hit_pcs.get_state().get_logs();
        assert!(hit_logs.has_parent());

        let hit_local_events = hit_logs.get_local_events();
        assert_eq!(5, hit_local_events.len());
        let hit_all_events = hit_logs.get_all_events();
        assert_eq!(10, hit_all_events.len());

        let expected_local_events = vec!(
            CombatEvent::AR(AttackResult::Hit),
            CombatEvent::Timing(CombatTiming::EndTurn(ParticipantId(0))),
            CombatEvent::Timing(CombatTiming::BeginTurn(ParticipantId(1))),
            CombatEvent::Timing(CombatTiming::EndTurn(ParticipantId(1))),
            CombatEvent::Timing(CombatTiming::EndRound(RoundId(1)))
        );
        assert_eq!(&expected_local_events, hit_local_events);

        let mut expected_events = vec!(
            CombatEvent::Timing(CombatTiming::BeginRound(RoundId(1))),
            CombatEvent::Timing(CombatTiming::BeginTurn(ParticipantId(0))),
            CombatEvent::AN(ActionName::AttackAction),
            CombatEvent::AN(ActionName::PrimaryAttack(AttackType::Normal)),
            CombatEvent::Attack(ParticipantId(0), ParticipantId(1))
        );
        assert_eq!(&expected_events, hit_logs.get_parent().get_local_events());

        expected_events.extend(expected_local_events.into_iter());
        assert_eq!(expected_events, hit_all_events);
    }

    #[test]
    fn fighter_vs_dummy_basic_attack_dmg() {
        let mut fighter = get_test_fighter_lvl_0();
        fighter.level_up(ClassName::Fighter, vec!(Box::new(FightingStyle(FightingStyles::GreatWeaponFighting)))).unwrap();
        let dummy = TargetDummy::new(isize::MAX, 14);

        let mut em: EM64 = EncounterManager::new();
        em.add_player(fighter.clone(), Box::new(BasicAttacks));
        em.add_participant(TeamMember::new(Team::Enemies, Box::new(dummy.clone())), ResourceManager::new(),Box::new(DoNothing));
        em.simulate_n_rounds(1).unwrap();

        let cs_rv = em.get_state_rv();
        let dmg_rv = cs_rv.get_dmg(ParticipantId(1));
        let atk_dmg: RV64 = fighter.get_basic_attack().unwrap().get_attack_dmg_rv(AttackHitType::Normal, dummy.get_ac(), dummy.get_resistances()).unwrap();
        assert_eq!(atk_dmg, dmg_rv);

        em.simulate_n_rounds(1).unwrap();
        let cs_rv = em.get_state_rv();
        let dmg_rv = cs_rv.get_dmg(ParticipantId(1));
        assert_eq!(atk_dmg.multiple(2), dmg_rv);
    }
}
