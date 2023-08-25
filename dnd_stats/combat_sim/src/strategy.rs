use character_builder::combat::{ActionName, ActionType, AttackType};
use character_builder::resources::ResourceName;
use character_builder::Square;
use crate::participant::{ParticipantId, TeamMember};
use crate::prob_combat_state::combat_state::CombatState;
use crate::prob_combat_state::combat_state::health::Health;

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

pub trait Strategy {
    fn get_action(&self, state: &CombatState, participants: &Vec<TeamMember>, me: ParticipantId) -> Option<StrategicOption>;
}

pub struct LinearStrategy {
    strategies: Vec<Box<dyn Strategy>>
}

impl LinearStrategy {
    pub fn new() -> Self {
        Self {
            strategies: Vec::new(),
        }
    }

    pub fn add_strategy(&mut self, str: Box<dyn Strategy>) {
        self.strategies.push(str);
    }
}

impl Strategy for LinearStrategy {
    fn get_action(&self, state: &CombatState, participants: &Vec<TeamMember>, me: ParticipantId) -> Option<StrategicOption> {
        for str in self.strategies.iter() {
            let so = str.get_action(state, participants, me);
            if so.is_some() {
                return so;
            }
        }
        None
    }
}

pub struct DoNothing;
impl Strategy for DoNothing {
    fn get_action(&self, _state: &CombatState, _participants: &Vec<TeamMember>, _me: ParticipantId) -> Option<StrategicOption> {
        None
    }
}

pub fn get_first_target(state: &CombatState, participants: &Vec<TeamMember>, me: ParticipantId) -> Option<Target> {
    let my_team = participants.get(me.0).unwrap().team;
    for i in 0..participants.len() {
        let pid = ParticipantId(i);
        if participants[i].team != my_team && state.is_alive(pid) {
            return Some(Target::Participant(pid))
        }
    }
    None
}

pub struct BasicAttackStr;
impl Strategy for BasicAttackStr {
    fn get_action(&self, state: &CombatState, participants: &Vec<TeamMember>, me: ParticipantId) -> Option<StrategicOption> {
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

pub struct SecondWindStr;
impl Strategy for SecondWindStr {
    fn get_action(&self, state: &CombatState, _: &Vec<TeamMember>, me: ParticipantId) -> Option<StrategicOption> {
        let my_rm = state.get_rm(me);
        let has_ba = my_rm.get_current(ResourceName::AT(ActionType::BonusAction)) > 0;
        let has_sw = my_rm.get_current(ResourceName::AN(ActionName::SecondWind)) > 0;
        if has_ba && has_sw && state.get_health(me) == Health::Bloodied {
            return Some(ActionName::SecondWind.into())
        }
        None
    }
}
