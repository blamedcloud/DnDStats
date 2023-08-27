use rand_var::rv_traits::RVError;

pub mod actions;
pub mod attack;
pub mod combat_event;
pub mod combat_state;
pub mod damage;
pub mod health;
pub mod movement;
pub mod participant;
pub mod resources;
pub mod strategy;
pub mod transposition;
pub mod triggers;

#[derive(Debug, Clone)]
pub enum CCError {
    ParticipantSizeErr,
    PMPushAfterCompile,
    PMNotCompiled,
    SMNotCompiled,
    SMPushAfterCompile,
    RVE(RVError),
    Other(String),
}

impl From<RVError> for CCError {
    fn from(value: RVError) -> Self {
        CCError::RVE(value)
    }
}

impl From<String> for CCError {
    fn from(value: String) -> Self {
        CCError::Other(value)
    }
}