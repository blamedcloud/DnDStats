use crate::actions::{ActionName, ActionType};

#[derive(Debug, Copy, Clone)]
pub enum TriggerType {
    WasHit,
    SuccessfulAttack,
}

pub struct TriggeredAction {
    action_type: ActionType,
    action_response: ActionName,

}


pub struct TriggerManager {

}
