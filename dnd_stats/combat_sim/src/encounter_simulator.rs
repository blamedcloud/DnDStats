use std::collections::HashMap;

use num::{BigRational, Rational64};

use combat_core::actions::{ActionName, ActionType, CombatAction};
use combat_core::attack::{Attack, AttackResult};
use combat_core::CCError;
use combat_core::combat_event::{CombatEvent, CombatTiming};
use combat_core::conditions::{ConditionLifetime, ConditionName};
use combat_core::damage::DamageTerm;
use combat_core::damage::dice_expr::DiceExpr;
use combat_core::participant::{Participant, ParticipantId, ParticipantManager, Team};
use combat_core::resources::{ResourceActionType, ResourceName};
use combat_core::resources::resource_amounts::ResourceCount;
use combat_core::skills::{ContestResult, SkillContest, SkillName};
use combat_core::strategy::{StrategicAction, Strategy, StrategyDecision, StrategyManager, Target};
use combat_core::triggers::{TriggerAction, TriggerContext, TriggerResponse, TriggerType};
use rand_var::map_rand_var::MapRandVar;
use rand_var::rand_var::prob_type::RVProb;
use rand_var::vec_rand_var::VecRandVar;

use crate::combat_state_rv::CombatStateRV;
use crate::combat_state_rv::prob_combat_state::ProbCombatState;
use crate::CSError;

pub enum HandledAction<'pm, P: RVProb> {
    InPlace(ProbCombatState<'pm, P>),
    Children(Vec<ProbCombatState<'pm, P>>)
}

type ResultHA<'pm, P> = Result<HandledAction<'pm, P>, CSError>;
type ResultCSE = Result<(), CSError>;

pub struct EncounterSimulator<'sm ,'pm, P: RVProb> {
    participants: &'pm ParticipantManager,
    strategies: &'sm StrategyManager<'pm>,
    round_num: u8,
    cs_rv: CombatStateRV<'pm, P>,
    merge_transpositions: bool,
}

pub type ES64<'sm, 'pm> = EncounterSimulator<'sm, 'pm, Rational64>;
pub type ESBig<'sm, 'pm> = EncounterSimulator<'sm, 'pm, BigRational>;

impl<'sm, 'pm, P: RVProb> EncounterSimulator<'sm, 'pm, P> {
    pub fn new(sm: &'sm StrategyManager<'pm>) -> Result<Self, CSError> {
        if !sm.is_compiled() {
            return Err(CSError::CCE(CCError::SMNotCompiled));
        }
        let pm = sm.get_pm();
        Ok(Self {
            participants: pm,
            strategies: sm,
            round_num: 0,
            cs_rv: CombatStateRV::new(pm),
            merge_transpositions: false,
        })
    }

    pub fn set_do_merges(&mut self, merges: bool) {
        self.merge_transpositions = merges
    }

    pub fn simulate_n_rounds(&mut self, n: u8) -> ResultCSE {
        if self.round_num == 0 {
            self.register_timing(CombatTiming::EncounterBegin);
        }
        for _ in 0..n {
            self.round_num += 1;
            self.register_timing(CombatTiming::BeginRound(self.round_num.into()));
            self.simulate_round()?;
            self.register_timing(CombatTiming::EndRound(self.round_num.into()));
            self.handle_merges();
        }
        Ok(())
    }

    pub fn get_state_rv(&self) -> &CombatStateRV<P> {
        &self.cs_rv
    }

    pub fn num_participants(&self) -> usize {
        self.participants.len()
    }

    pub fn get_team(&self, pid: ParticipantId) -> Team {
        self.participants.get_participant(pid).team
    }

    fn handle_merges(&mut self) {
        if self.merge_transpositions {
            self.cs_rv.merge_states();
        }
    }

    fn register_timing(&mut self, ct: CombatTiming) {
        let event = ct.into();
        for pcs in self.cs_rv.get_states_mut() {
            if pcs.is_valid_timing(ct) {
                pcs.push(event);

                for i in 0..self.participants.len() {
                    let pid = ParticipantId(i);
                    let ort = ct.get_refresh_timing(pid);
                    ort.map(|rt| pcs.handle_refresh(pid, rt));
                }
            }
        }
    }

    fn simulate_round(&mut self) -> ResultCSE {
        for i in 0..self.participants.len() {
            let pid = ParticipantId(i);
            self.register_timing(CombatTiming::BeginTurn(pid));
            self.simulate_turn(pid)?;
            self.register_timing(CombatTiming::EndTurn(pid));
            self.handle_merges();
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

    fn get_strategy(&self, pid: ParticipantId) -> &Box<dyn Strategy + 'pm> {
        self.strategies.get_strategy(pid)
    }

    fn is_combat_over(&self, pcs: &mut ProbCombatState<'pm, P>) -> bool {
        if pcs.get_state().get_last_combat_timing().unwrap() == CombatTiming::EncounterEnd {
            return true;
        }
        let mut player_alive = false;
        let mut enemy_alive = false;
        for i in 0..self.num_participants() {
            let pid = ParticipantId(i);
            if pcs.is_alive(pid) {
                match self.get_team(pid) {
                    Team::Players => player_alive = true,
                    Team::Enemies => enemy_alive = true,
                }
            }
            if player_alive && enemy_alive {
                return false;
            }
        }
        pcs.push(CombatEvent::Timing(CombatTiming::EncounterEnd));
        true
    }

    fn finish_turn(&self, mut pcs: ProbCombatState<'pm, P>, pid: ParticipantId) -> Result<Vec<ProbCombatState<'pm, P>>, CSError> {
        if self.is_combat_over(&mut pcs) || pcs.is_dead(pid) {
            return Ok(vec!(pcs));
        }
        let strategy = self.get_strategy(pid);
        let sd = strategy.get_action(pcs.get_state());
        if let StrategyDecision::MyAction(so) = sd {
            if self.possible_action(&pcs, pid, so) {
                let children = self.finish_action(pcs, pid, so)?;
                let mut finished_pcs = Vec::new();
                for pcs in children.into_iter() {
                    let new_pcs = self.finish_turn(pcs, pid)?;
                    finished_pcs.extend(new_pcs.into_iter());
                }
                Ok(finished_pcs)
            } else {
                // strategy gave me an invalid StrategicDecision
                Ok(vec!(pcs))
            }
        } else {
            if let StrategyDecision::RemoveCondition(cond, at) = sd {
                // remove a condition
                if self.possible_condition_removal(&pcs, pid, cond, at) {
                    pcs.remove_condition(pid, cond, at);
                    Ok(self.finish_turn(pcs, pid)?)
                } else {
                    // invalid StrategicDecision
                    Ok(vec!(pcs))
                }
            } else {
                // strategy end-turn on purpose.
                Ok(vec!(pcs))
            }
        }
    }

    fn possible_condition_removal(&self, pcs: &ProbCombatState<'pm, P>, pid: ParticipantId, cn: ConditionName, at: ActionType) -> bool {
        let cm = pcs.get_state().get_cm(pid);
        let has_cond = cm.has_condition(&cn);
        if !has_cond {
            return false;
        }
        let has_lifetime = cm.has_lifetime(&ConditionLifetime::UntilSpendAT(at));
        if !has_lifetime {
            return false;
        }
        let cns = cm.get_cns_for_lifetime(&ConditionLifetime::UntilSpendAT(at));
        if !cns.contains(&cn) {
            return false;
        }
        let rm = pcs.get_rm(pid);
        if rm.has_resource(at.into()) {
            match at {
                ActionType::Movement => {
                    // assume takes full movement
                    rm.is_full(ResourceName::Movement)
                }
                ActionType::HalfMove => {
                    let half_move: ResourceCount = (rm.get_cap(ResourceName::Movement) / 2).unwrap().into();
                    half_move.is_uncapped() || rm.get_current(ResourceName::Movement) >= half_move.count().unwrap()
                },
                _ => {
                    rm.get_current(at.into()) > 0
                }
            }
        } else {
            false
        }
    }

    fn get_participant(&self, pid: ParticipantId) -> &Box<dyn Participant> {
        &self.participants.get_participant(pid).participant
    }

    fn is_dead_at_zero(&self, pid: ParticipantId) -> bool {
        self.get_team(pid) == Team::Enemies
    }

    fn possible_action(&self, pcs: &ProbCombatState<'pm, P>, pid: ParticipantId, so: StrategicAction) -> bool {
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
                if rm.has_resource(at.into()) && rm.get_current(at.into()) == 0 {
                    has_resources = false;
                }
            }
        }
        has_action && valid_target && has_resources
    }

    fn finish_action(&self, pcs: ProbCombatState<'pm, P>, pid: ParticipantId, so: StrategicAction) -> Result<Vec<ProbCombatState<'pm, P>>, CSError> {
        let handled_action = self.handle_action(pcs, pid, so)?;
        match handled_action {
            HandledAction::InPlace(p) => Ok(vec!(p)),
            HandledAction::Children(v) => Ok(v),
        }
    }

    fn handle_action(&self, mut pcs: ProbCombatState<'pm, P>, pid: ParticipantId, so: StrategicAction) -> ResultHA<'pm, P> {
        let an = so.action_name;
        let participant = self.get_participant(pid);
        let co = participant.get_action_manager().get(&an).unwrap();
        let at = co.action_type;
        pcs.spend_action_resources(pid, an, at);

        pcs.push(CombatEvent::AN(an));

        match &co.action {
            CombatAction::Attack(atk) => {
                if let Target::Participant(target_pid) = so.target.unwrap() {
                    pcs.push(CombatEvent::Attack(pid, target_pid));
                    Ok(HandledAction::Children(self.handle_attack(pcs, atk, pid, target_pid)?))
                } else {
                    Err(CSError::InvalidTarget)
                }
            },
            CombatAction::SelfHeal(de) => {
                let heal: VecRandVar<P> = de.get_heal_rv()?;
                Ok(HandledAction::Children(pcs.add_dmg(&heal, pid, self.is_dead_at_zero(pid))))
            },
            CombatAction::AdditionalAttacks(aa) => {
                pcs.get_rm_mut(pid).gain(ResourceName::RAT(ResourceActionType::SingleAttack), *aa as usize);
                Ok(HandledAction::InPlace(pcs))
            },
            CombatAction::ByName => {
                match an {
                    ActionName::ActionSurge => {
                        pcs.get_rm_mut(pid).gain(ResourceName::RAT(ResourceActionType::Action), 1);
                        Ok(HandledAction::InPlace(pcs))
                    },
                    ActionName::ShoveProne => {
                        if let Target::Participant(target_pid) = so.target.unwrap() {
                            Ok(HandledAction::Children(self.handle_shove_prone(pcs, pid, target_pid)?))
                        } else {
                            Err(CSError::InvalidTarget)
                        }
                    }
                    _ => Err(CSError::ActionNotHandled),
                }
            }
        }
    }

    fn handle_shove_prone(&self, mut pcs: ProbCombatState<'pm, P>, shover_pid: ParticipantId, target_pid: ParticipantId) -> Result<Vec<ProbCombatState<'pm, P>>, CSError> {
        let shover = self.get_participant(shover_pid);
        let target = self.get_participant(target_pid);
        let t_sm = target.get_skill_manager();
        let t_skill = t_sm.choose_grapple_defense(target.get_ability_scores(), target.get_prof());
        pcs.push(CombatEvent::SkillContest(shover_pid, SkillName::Athletics, target_pid, t_skill));
        let contest = SkillContest::build(shover, target, SkillName::Athletics, t_skill);
        let ce_skc = contest.result.map_keys(|cr| CombatEvent::SkCR(cr));
        let children = pcs.split(ce_skc);
        let mut results = Vec::with_capacity(children.len());
        for mut child in children {
            if let CombatEvent::SkCR(cr) = child.get_last_event().unwrap() {
                match cr {
                    ContestResult::InitiatorWins => {
                        child.apply_default_condition(target_pid, ConditionName::Prone);
                        results.push(child);
                    }
                    ContestResult::DefenderWins => results.push(child)
                };
            } else {
                return Err(CSError::UnknownEvent(child.get_last_event().unwrap()));
            }
        }
        Ok(results)
    }

    fn handle_attack(&self, pcs: ProbCombatState<'pm, P>, atk: &impl Attack, atker_pid: ParticipantId, target_pid: ParticipantId) -> Result<Vec<ProbCombatState<'pm, P>>, CSError> {
        let attacker = self.get_participant(atker_pid);
        let target = self.get_participant(target_pid);
        let dead_at_zero = self.is_dead_at_zero(target_pid);
        let atk_cm = pcs.get_state().get_cm(atker_pid);
        let target_cm = pcs.get_state().get_cm(target_pid);
        let roll_type = atk_cm.overall_atk_mod(target_cm, atk.get_atk_range());
        let ce_rv: MapRandVar<CombatEvent, P> = atk.get_ce_rv(roll_type, target.get_ac())?;
        // TODO: handle AC boosts somehow...

        if attacker.has_triggers() && attacker.get_trigger_manager().unwrap().has_triggers(TriggerType::SuccessfulAttack) {
            let children = pcs.split(ce_rv);
            let mut results = Vec::with_capacity(children.len());
            let resist = target.get_resistances();
            for mut child in children {
                if let CombatEvent::AR(ar) = child.get_last_event().unwrap() {
                    match ar {
                        AttackResult::Miss => {
                            let v = child.add_dmg(&atk.get_miss_dmg(resist, vec!())?, target_pid, dead_at_zero);
                            results.extend(v.into_iter());
                        }
                        _ => {
                            let response = self.get_strategy(atker_pid).handle_trigger(TriggerType::SuccessfulAttack, TriggerContext::AR(ar), child.get_state());
                            let cost = self.validate_trigger_cost(&child, atker_pid, TriggerType::SuccessfulAttack, &response);
                            if cost.is_some() {
                                child.spend_resource_cost(atker_pid, cost.unwrap());
                                let bonus_dmg = self.handle_dmg_bonus(response);
                                let v = child.add_dmg(&atk.get_ar_dmg(ar, resist, bonus_dmg)?, target_pid, dead_at_zero);
                                results.extend(v.into_iter());
                            } else {
                                return Err(CSError::InvalidTriggerResponse);
                            }
                        }
                    }
                } else {
                    return Err(CSError::UnknownEvent(child.get_last_event().unwrap()));
                }
            }
            Ok(results)
        } else {
            let ce_dmg_map = atk.get_dmg_map(target.get_resistances())?.into_ce_map();
            Ok(pcs.split_dmg(ce_rv, ce_dmg_map, target_pid, dead_at_zero))
        }
    }

    fn handle_dmg_bonus(&self, response: Vec<TriggerResponse>) -> Vec<DamageTerm> {
        let mut v = Vec::with_capacity(response.len());
        for tr in response {
            if let TriggerAction::AddAttackDamage(dt) = tr.action {
                v.push(dt);
            }
        }
        v
    }

    fn validate_trigger_cost(&self, pcs: &ProbCombatState<'pm, P>, pid: ParticipantId, tt: TriggerType, response: &Vec<TriggerResponse>) -> Option<HashMap<ResourceName, usize>> {
        match tt {
            TriggerType::WasHit => todo!(),
            TriggerType::SuccessfulAttack => {
                let mut resource_cost: HashMap<ResourceName, usize> = HashMap::new();
                for tr in response.iter() {
                    if let TriggerAction::AddAttackDamage(_) = &tr.action {
                        for rn in &tr.resources {
                            resource_cost.entry(*rn)
                                .and_modify(|count| *count += 1)
                                .or_insert(1);
                        }
                    } else {
                        return None;
                    }
                }
                if pcs.get_rm(pid).check_counts(&resource_cost) {
                    Some(resource_cost)
                } else {
                    None
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use num::{One, Rational64};

    use character_builder::Character;
    use character_builder::classes::ClassName;
    use character_builder::equipment::{Armor, Equipment, OffHand, Weapon};
    use character_builder::feature::fighting_style::{FightingStyle, FightingStyles};
    use combat_core::ability_scores::AbilityScores;
    use combat_core::actions::{ActionName, AttackType};
    use combat_core::attack::AttackResult;
    use combat_core::attack::basic_attack::BasicAttack;
    use combat_core::combat_event::{CombatEvent, CombatTiming, RoundId};
    use combat_core::D20RollType;
    use combat_core::damage::{DamageDice, DamageType};
    use combat_core::health::Health;
    use combat_core::participant::{Participant, ParticipantId, ParticipantManager};
    use combat_core::strategy::basic_atk_str::BasicAtkStrBuilder;
    use combat_core::strategy::basic_strategies::DoNothingBuilder;
    use combat_core::strategy::linear_str::PairStrBuilder;
    use combat_core::strategy::second_wind_str::SecondWindStrBuilder;
    use combat_core::strategy::StrategyManager;
    use rand_var::vec_rand_var::VRV64;
    use rand_var::rand_var::RandVar;

    use crate::encounter_simulator::{EncounterSimulator, ES64};
    use crate::monster::Monster;
    use crate::player::Player;
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

        let player = Player::from(fighter);
        let mut pm = ParticipantManager::new();
        pm.add_player(Box::new(player)).unwrap();
        pm.add_enemy(Box::new(dummy)).unwrap();
        pm.compile();

        let mut sm = StrategyManager::new(&pm).unwrap();
        sm.add_participant(DoNothingBuilder).unwrap();
        sm.add_participant(DoNothingBuilder).unwrap();

        let mut em: ES64 = EncounterSimulator::new(&sm).unwrap();
        em.simulate_n_rounds(1).unwrap();

        {
            let cs_rv = em.get_state_rv();
            assert_eq!(1, cs_rv.len());

            let pcs = cs_rv.get_pcs(0);
            assert_eq!(Rational64::one(), *pcs.get_prob());

            let logs = pcs.get_state().get_logs();
            assert!(!logs.has_parent());

            let events = logs.get_local_events();
            assert_eq!(7, events.len());
            let all_events = logs.get_all_events();
            assert_eq!(all_events, events.clone());

            let expected_events = vec!(
                CombatEvent::Timing(CombatTiming::EncounterBegin),
                CombatEvent::Timing(CombatTiming::BeginRound(RoundId(1))),
                CombatEvent::Timing(CombatTiming::BeginTurn(ParticipantId(0))),
                CombatEvent::Timing(CombatTiming::EndTurn(ParticipantId(0))),
                CombatEvent::Timing(CombatTiming::BeginTurn(ParticipantId(1))),
                CombatEvent::Timing(CombatTiming::EndTurn(ParticipantId(1))),
                CombatEvent::Timing(CombatTiming::EndRound(RoundId(1)))
            );
            assert_eq!(all_events, expected_events);
        }
    }

    #[test]
    fn fighter_vs_dummy_basic_attack() {
        let mut fighter = get_test_fighter_lvl_0();
        fighter.level_up(ClassName::Fighter, vec!(Box::new(FightingStyle(FightingStyles::GreatWeaponFighting)))).unwrap();
        let dummy = TargetDummy::new(isize::MAX, 14);

        let player = Player::from(fighter.clone());
        let mut pm = ParticipantManager::new();
        pm.add_player(Box::new(player)).unwrap();
        pm.add_enemy(Box::new(dummy.clone())).unwrap();
        pm.compile();

        let mut sm = StrategyManager::new(&pm).unwrap();
        sm.add_participant(BasicAtkStrBuilder).unwrap();
        sm.add_participant(DoNothingBuilder).unwrap();

        let mut em: ES64 = EncounterSimulator::new(&sm).unwrap();
        em.simulate_n_rounds(1).unwrap();

        {
            let cs_rv = em.get_state_rv();
            assert_eq!(3, cs_rv.len());

            let index_rv = cs_rv.get_index_rv();
            let ar_rv: VRV64 = fighter.get_weapon_attack().unwrap()
                .get_attack_result_rv(D20RollType::Normal, dummy.get_ac()).unwrap()
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
            assert_eq!(11, hit_all_events.len());

            let expected_local_events = vec!(
                CombatEvent::AR(AttackResult::Hit),
                CombatEvent::Timing(CombatTiming::EndTurn(ParticipantId(0))),
                CombatEvent::Timing(CombatTiming::BeginTurn(ParticipantId(1))),
                CombatEvent::Timing(CombatTiming::EndTurn(ParticipantId(1))),
                CombatEvent::Timing(CombatTiming::EndRound(RoundId(1)))
            );
            assert_eq!(&expected_local_events, hit_local_events);

            let mut expected_events = vec!(
                CombatEvent::Timing(CombatTiming::EncounterBegin),
                CombatEvent::Timing(CombatTiming::BeginRound(RoundId(1))),
                CombatEvent::Timing(CombatTiming::BeginTurn(ParticipantId(0))),
                CombatEvent::AN(ActionName::AttackAction),
                CombatEvent::AN(ActionName::PrimaryAttack(AttackType::Normal)),
                CombatEvent::Attack(ParticipantId(0), ParticipantId(1))
            );
            assert_eq!(&expected_events, hit_logs.get_first_parent().unwrap().get_local_events());

            expected_events.extend(expected_local_events.into_iter());
            assert_eq!(expected_events, hit_all_events);
        }
    }

    #[test]
    fn fighter_vs_orc_stats() {
        let mut fighter = get_test_fighter_lvl_0();
        fighter.level_up(ClassName::Fighter, vec!(Box::new(FightingStyle(FightingStyles::GreatWeaponFighting)))).unwrap();
        let orc = TargetDummy::new(15, 13);

        let player = Player::from(fighter.clone());
        let mut pm = ParticipantManager::new();
        pm.add_player(Box::new(player)).unwrap();
        pm.add_enemy(Box::new(orc.clone())).unwrap();
        pm.compile();

        let mut sm = StrategyManager::new(&pm).unwrap();
        sm.add_participant(BasicAtkStrBuilder).unwrap();
        sm.add_participant(DoNothingBuilder).unwrap();

        let mut em: ES64 = EncounterSimulator::new(&sm).unwrap();
        em.simulate_n_rounds(1).unwrap();

        {
            let cs_rv = em.get_state_rv();
            // miss, hit, hit -> bloody, hit -> kill, crit, crit -> bloody, crit -> kill
            assert_eq!(7, cs_rv.len());

            let orc_pid = ParticipantId(1);
            let orc_bld = Health::calc_bloodied(orc.get_max_hp());
            assert_eq!(8, orc_bld);

            let miss = cs_rv.get_pcs(0).get_dmg(orc_pid);
            assert_eq!(0, miss.lower_bound());
            assert_eq!(0, miss.upper_bound());

            let hit_hlt = cs_rv.get_pcs(1).get_dmg(orc_pid);
            assert_eq!(5, hit_hlt.lower_bound());
            assert_eq!(orc_bld - 1, hit_hlt.upper_bound());

            let hit_bld = cs_rv.get_pcs(2).get_dmg(orc_pid);
            assert_eq!(orc_bld, hit_bld.lower_bound());
            assert_eq!(orc.get_max_hp() - 1, hit_bld.upper_bound());

            let hit_die = cs_rv.get_pcs(3).get_dmg(orc_pid);
            assert_eq!(orc.get_max_hp(), hit_die.lower_bound());
            assert_eq!(orc.get_max_hp(), hit_die.upper_bound());

            let crit_hlt = cs_rv.get_pcs(4).get_dmg(orc_pid);
            assert_eq!(7, crit_hlt.lower_bound());
            assert_eq!(orc_bld - 1, crit_hlt.upper_bound());

            let crit_bld = cs_rv.get_pcs(5).get_dmg(orc_pid);
            assert_eq!(orc_bld, crit_bld.lower_bound());
            assert_eq!(orc.get_max_hp() - 1, crit_bld.upper_bound());

            let crit_die = cs_rv.get_pcs(6).get_dmg(orc_pid);
            assert_eq!(orc.get_max_hp(), crit_die.lower_bound());
            assert_eq!(orc.get_max_hp(), crit_die.upper_bound());

            let dmg_rv = cs_rv.get_dmg(orc_pid);
            let atk_dmg: VRV64 = fighter
                .get_weapon_attack().unwrap()
                .get_attack_dmg_rv(D20RollType::Normal, orc.get_ac(), orc.get_resistances()).unwrap()
                .cap_ub(orc.get_max_hp()).unwrap();
            assert_eq!(atk_dmg, dmg_rv);
        }
    }

    #[test]
    fn fighter_vs_orc_merged() {
        let mut fighter = get_test_fighter_lvl_0();
        fighter.level_up(ClassName::Fighter, vec!(Box::new(FightingStyle(FightingStyles::GreatWeaponFighting)))).unwrap();
        let orc = TargetDummy::new(15, 13);

        let player = Player::from(fighter.clone());
        let mut pm = ParticipantManager::new();
        pm.add_player(Box::new(player)).unwrap();
        pm.add_enemy(Box::new(orc.clone())).unwrap();
        pm.compile();

        let mut sm = StrategyManager::new(&pm).unwrap();
        sm.add_participant(BasicAtkStrBuilder).unwrap();
        sm.add_participant(DoNothingBuilder).unwrap();

        let mut em: ES64 = EncounterSimulator::new(&sm).unwrap();
        em.set_do_merges(true);
        em.simulate_n_rounds(1).unwrap();

        let cs_rv = em.get_state_rv();
        //healthy, bloodied, killed
        assert_eq!(3, cs_rv.len());

        let orc_pid = ParticipantId(1);
        let dmg_rv = cs_rv.get_dmg(orc_pid);
        let wa = fighter.get_weapon_attack().unwrap();
        let atk_dmg: VRV64 = wa
            .get_attack_dmg_rv(D20RollType::Normal, orc.get_ac(), orc.get_resistances()).unwrap()
            .cap_ub(orc.get_max_hp()).unwrap();
        assert_eq!(atk_dmg, dmg_rv);
    }

    #[test]
    fn fighter_vs_orc_strategy() {
        let mut fighter = get_test_fighter_lvl_0();
        fighter.level_up(ClassName::Fighter, vec!(Box::new(FightingStyle(FightingStyles::GreatWeaponFighting)))).unwrap();
        let fighter_str = PairStrBuilder::new(BasicAtkStrBuilder, SecondWindStrBuilder);
        let player = Player::from(fighter.clone());

        let ba = BasicAttack::new(5, DamageType::Slashing, 3, DamageDice::D12, 1);
        let orc = Monster::new(15, 13, 2, ba, 1);

        let mut pm = ParticipantManager::new();
        pm.add_enemy(Box::new(orc.clone())).unwrap();
        pm.add_player(Box::new(player)).unwrap();
        pm.compile();

        let mut sm = StrategyManager::new(&pm).unwrap();
        sm.add_participant(BasicAtkStrBuilder).unwrap();
        sm.add_participant(fighter_str).unwrap();

        let mut em: ES64 = EncounterSimulator::new(&sm).unwrap();
        em.simulate_n_rounds(1).unwrap();

        {
            let cs_rv = em.get_state_rv();
            assert_eq!(59, cs_rv.len());

            let branch = cs_rv.get_pcs(16);
            let full_log = branch.get_state().get_logs().get_all_events();

            let expected_events = vec!(
                CombatEvent::Timing(CombatTiming::EncounterBegin),
                CombatEvent::Timing(CombatTiming::BeginRound(RoundId(1))),
                CombatEvent::Timing(CombatTiming::BeginTurn(ParticipantId(0))),
                CombatEvent::AN(ActionName::AttackAction),
                CombatEvent::AN(ActionName::PrimaryAttack(AttackType::Normal)),
                CombatEvent::Attack(ParticipantId(0), ParticipantId(1)),
                CombatEvent::AR(AttackResult::Hit),
                CombatEvent::HP(ParticipantId(1), Health::Bloodied),
                CombatEvent::Timing(CombatTiming::EndTurn(ParticipantId(0))),
                CombatEvent::Timing(CombatTiming::BeginTurn(ParticipantId(1))),
                CombatEvent::AN(ActionName::AttackAction),
                CombatEvent::AN(ActionName::PrimaryAttack(AttackType::Normal)),
                CombatEvent::Attack(ParticipantId(1), ParticipantId(0)),
                CombatEvent::AR(AttackResult::Hit),
                CombatEvent::AN(ActionName::SecondWind),
                CombatEvent::HP(ParticipantId(1), Health::Healthy),
                CombatEvent::Timing(CombatTiming::EndTurn(ParticipantId(1))),
                CombatEvent::Timing(CombatTiming::EndRound(RoundId(1))),
            );
            assert_eq!(expected_events, full_log);
        }
    }
}
