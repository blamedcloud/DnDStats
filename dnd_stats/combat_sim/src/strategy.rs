use character_builder::combat::{ActionName, ActionType, AttackType};
use character_builder::resources::ResourceName;
use character_builder::Square;
use crate::combat_log::CombatState;
use crate::participant::{ParticipantId, TeamMember};

// TODO: add squares eventually (for spells and such)
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

pub struct DoNothing;
impl Strategy for DoNothing {
    fn get_action(&self, _state: &CombatState, _participants: &Vec<TeamMember>, _me: ParticipantId) -> Option<StrategicOption> {
        None
    }
}

pub fn get_first_target(me: ParticipantId, participants: &Vec<TeamMember>) -> Option<Target> {
    let my_team = participants.get(me.0).unwrap().team;
    for i in 0..participants.len() {
        if participants[i].team != my_team {
            return Some(Target::Participant(ParticipantId(i)))
        }
    }
    None
}

pub struct BasicAttacks;
impl Strategy for BasicAttacks {
    fn get_action(&self, state: &CombatState, participants: &Vec<TeamMember>, me: ParticipantId) -> Option<StrategicOption> {
        let my_rm = state.get_rm(me);
        if my_rm.get_current(ResourceName::AT(ActionType::Action)) > 0 {
            return Some(ActionName::AttackAction.into())
        }
        if my_rm.get_current(ResourceName::AT(ActionType::SingleAttack)) > 0 {
            let target = get_first_target(me, participants);
            return Some(StrategicOption {
                action_name: ActionName::PrimaryAttack(AttackType::Normal),
                target
            })
        }
        None
    }
}
