use character_builder::combat::ActionName;
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

pub trait Strategy {
    fn get_action(&self, state: &CombatState, participants: &Vec<TeamMember>, me: ParticipantId) -> Option<StrategicOption>;
}

pub struct DoNothing;
impl Strategy for DoNothing {
    fn get_action(&self, _state: &CombatState, _participants: &Vec<TeamMember>, _me: ParticipantId) -> Option<StrategicOption> {
        None
    }
}
