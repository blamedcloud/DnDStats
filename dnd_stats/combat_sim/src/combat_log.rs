use character_builder::combat::attack::AttackResult;
use character_builder::resources::RefreshTiming;
use crate::participant::ParticipantId;


#[derive(Debug, Ord, PartialOrd, PartialEq, Eq, Clone, Copy)]
pub enum CombatTiming {
    BeginRound,
    EndRound,
    BeginTurn(ParticipantId),
    EndTurn(ParticipantId),
}

impl CombatTiming {
    pub fn get_refresh_timing(&self, pid: ParticipantId) -> RefreshTiming {
        match self {
            CombatTiming::BeginRound => RefreshTiming::StartRound,
            CombatTiming::EndRound => RefreshTiming::EndRound,
            CombatTiming::BeginTurn(t_pid) => {
                if pid == *t_pid {
                    RefreshTiming::StartMyTurn
                } else {
                    RefreshTiming::StartOtherTurn
                }
            }
            CombatTiming::EndTurn(t_pid) => {
                if pid == *t_pid {
                    RefreshTiming::EndMyTurn
                } else {
                    RefreshTiming::EndOtherTurn
                }
            }
        }
    }
}

#[derive(Debug, Ord, PartialOrd, PartialEq, Eq, Clone, Copy)]
pub enum CombatEvent {
    Timing(CombatTiming),
    Attack(ParticipantId, ParticipantId),
    AR(AttackResult),
}

pub type CombatLog = Vec<CombatEvent>;
