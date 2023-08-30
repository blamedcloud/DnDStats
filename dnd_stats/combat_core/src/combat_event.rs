use crate::actions::ActionName;
use crate::attack::AttackResult;
use crate::conditions::ConditionName;
use crate::health::Health;
use crate::participant::ParticipantId;
use crate::resources::RefreshTiming;
use crate::skills::{ContestResult, SkillName};

#[derive(Debug, Ord, PartialOrd, PartialEq, Eq, Hash, Clone, Copy)]
pub struct RoundId(pub u8);
impl From<u8> for RoundId {
    fn from(value: u8) -> Self {
        RoundId(value)
    }
}

#[derive(Debug, Ord, PartialOrd, PartialEq, Eq, Hash, Clone, Copy)]
pub enum CombatTiming {
    EncounterBegin,
    EncounterEnd,
    BeginRound(RoundId),
    EndRound(RoundId),
    BeginTurn(ParticipantId),
    EndTurn(ParticipantId),
}

impl CombatTiming {
    pub fn get_refresh_timing(&self, pid: ParticipantId) -> Option<RefreshTiming> {
        match self {
            CombatTiming::EncounterBegin => None,
            CombatTiming::EncounterEnd => None,
            CombatTiming::BeginRound(_) => Some(RefreshTiming::StartRound),
            CombatTiming::EndRound(_) => Some(RefreshTiming::EndRound),
            CombatTiming::BeginTurn(t_pid) => {
                if pid == *t_pid {
                    Some(RefreshTiming::StartMyTurn)
                } else {
                    Some(RefreshTiming::StartOtherTurn)
                }
            },
            CombatTiming::EndTurn(t_pid) => {
                if pid == *t_pid {
                    Some(RefreshTiming::EndMyTurn)
                } else {
                    Some(RefreshTiming::EndOtherTurn)
                }
            },
        }
    }
}

#[derive(Debug, Ord, PartialOrd, PartialEq, Eq, Clone, Copy)]
pub enum CombatEvent {
    Timing(CombatTiming),
    AN(ActionName),
    Attack(ParticipantId, ParticipantId),
    AR(AttackResult),
    HP(ParticipantId, Health),
    ApplyCond(ConditionName, ParticipantId),
    RemoveCond(ConditionName),
    SkillContest(ParticipantId, SkillName, ParticipantId, SkillName),
    SkCR(ContestResult),
}

impl From<AttackResult> for CombatEvent {
    fn from(value: AttackResult) -> Self {
        CombatEvent::AR(value)
    }
}

impl From<CombatTiming> for CombatEvent {
    fn from(value: CombatTiming) -> Self {
        CombatEvent::Timing(value)
    }
}

impl From<ActionName> for CombatEvent {
    fn from(value: ActionName) -> Self {
        CombatEvent::AN(value)
    }
}
