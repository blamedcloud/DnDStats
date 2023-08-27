use std::fmt::Debug;
use rand_var::rv_traits::prob_type::RVProb;
use crate::actions::{ActionName, ActionType, AttackType};
use crate::combat_state::CombatState;
use crate::health::Health;
use crate::movement::Square;
use crate::participant::{ParticipantId, TeamMember};
use crate::resources::ResourceName;

// TODO: add shapes eventually (for spells and such)
#[derive(Debug, Ord, PartialOrd, PartialEq, Eq, Clone, Copy)]
pub enum Target {
    Participant(ParticipantId),
    Tile(Square, Square),
}

#[derive(Debug, Ord, PartialOrd, PartialEq, Eq, Clone, Copy)]
pub struct StrategicOption {
    pub action_name: ActionName,
    pub target: Option<Target>
}

impl From<ActionName> for StrategicOption {
    fn from(value: ActionName) -> Self {
        Self {
            action_name: value,
            target: None,
        }
    }
}

pub trait Strategy<T: RVProb, E> : Debug {
    fn get_action(&self, state: &CombatState, participants: &Vec<TeamMember<T, E>>, me: ParticipantId) -> Option<StrategicOption>;
}

#[derive(Debug)]
pub struct LinearStrategy<T: RVProb, E: Debug> {
    strategies: Vec<Box<dyn Strategy<T, E>>>
}

impl<T: RVProb, E: Debug> LinearStrategy<T, E> {
    pub fn new() -> Self {
        Self {
            strategies: Vec::new(),
        }
    }

    pub fn add_strategy(&mut self, str: Box<dyn Strategy<T, E>>) {
        self.strategies.push(str);
    }
}

impl<T: RVProb + Debug, E: Debug> Strategy<T, E> for LinearStrategy<T, E> {
    fn get_action(&self, state: &CombatState, participants: &Vec<TeamMember<T, E>>, me: ParticipantId) -> Option<StrategicOption> {
        for str in self.strategies.iter() {
            let so = str.get_action(state, participants, me);
            if so.is_some() {
                return so;
            }
        }
        None
    }
}

#[derive(Debug)]
pub struct DoNothing;
impl<T: RVProb, E> Strategy<T, E> for DoNothing {
    fn get_action(&self, _: &CombatState, _: &Vec<TeamMember<T, E>>, _: ParticipantId) -> Option<StrategicOption> {
        None
    }
}

pub fn get_first_target<T: RVProb, E>(state: &CombatState, participants: &Vec<TeamMember<T, E>>, me: ParticipantId) -> Option<Target> {
    let my_team = participants.get(me.0).unwrap().team;
    for i in 0..participants.len() {
        let pid = ParticipantId(i);
        if participants[i].team != my_team && state.is_alive(pid) {
            return Some(Target::Participant(pid))
        }
    }
    None
}

#[derive(Debug)]
pub struct BasicAttackStr;
impl<T:RVProb, E> Strategy<T, E> for BasicAttackStr {
    fn get_action(&self, state: &CombatState, participants: &Vec<TeamMember<T, E>>, me: ParticipantId) -> Option<StrategicOption> {
        let my_rm = state.get_rm(me);
        if my_rm.get_current(ResourceName::AT(ActionType::Action)) > 0 {
            return Some(ActionName::AttackAction.into())
        }
        if my_rm.get_current(ResourceName::AT(ActionType::SingleAttack)) > 0 {
            let target = get_first_target(state, participants, me);
            return Some(StrategicOption {
                action_name: ActionName::PrimaryAttack(AttackType::Normal),
                target
            })
        }
        None
    }
}

#[derive(Debug)]
pub struct SecondWindStr;
impl<T:RVProb, E> Strategy<T, E> for SecondWindStr {
    fn get_action(&self, state: &CombatState, _: &Vec<TeamMember<T, E>>, me: ParticipantId) -> Option<StrategicOption> {
        let my_rm = state.get_rm(me);
        let has_ba = my_rm.get_current(ResourceName::AT(ActionType::BonusAction)) > 0;
        let has_sw = my_rm.get_current(ResourceName::AN(ActionName::SecondWind)) > 0;
        if has_ba && has_sw && state.get_health(me) == Health::Bloodied {
            return Some(ActionName::SecondWind.into())
        }
        None
    }
}
