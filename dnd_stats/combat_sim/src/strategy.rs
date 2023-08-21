use character_builder::combat::ActionName;
use crate::combat_log::CombatLog;
use crate::participant::{ParticipantId, TeamMember};

// TODO: add squares eventually (for spells and such)
#[derive(Debug, Ord, PartialOrd, PartialEq, Eq, Clone, Copy)]
pub enum Target {
    Participant(ParticipantId),
}

#[derive(Debug, Ord, PartialOrd, PartialEq, Eq, Clone, Copy)]
pub struct StrategicOption {
    pub action_name: ActionName,
    pub target: Option<Target>
}

pub trait Strategy {
    fn get_action(&self, log: &CombatLog, participants: &Vec<TeamMember>, me: ParticipantId) -> Option<StrategicOption>;
}

pub struct DoNothing;
impl Strategy for DoNothing {
    fn get_action(&self, _log: &CombatLog, _participants: &Vec<TeamMember>, _me: ParticipantId) -> Option<StrategicOption> {
        None
    }
}
