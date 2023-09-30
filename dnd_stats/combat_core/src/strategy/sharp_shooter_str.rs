use crate::actions::{ActionName, AttackType};
use crate::combat_state::CombatState;
use crate::participant::{ParticipantId, TeamMember};
use crate::resources::{ResourceActionType, ResourceName};
use crate::strategy::{StrategicAction, Strategy, StrategyBuilder, StrategyDecision};
use crate::triggers::{TriggerInfo, TriggerResponse};

pub struct SharpShooterStrBldr {
    use_ss: bool // dyn Fn(isize) -> bool,
}
impl SharpShooterStrBldr {
    pub fn new(use_ss: bool) -> Self {
        Self {
            use_ss,
        }
    }
}
impl StrategyBuilder for SharpShooterStrBldr {
    fn build_strategy<'pm>(&self, participants: &'pm Vec<TeamMember>, me: ParticipantId) -> Box<dyn Strategy + 'pm> {
        let str = SharpShooterStr {
            participants,
            my_pid: me,
            use_ss: self.use_ss,
        };
        Box::new(str)
    }
}

#[derive(Debug)]
pub struct SharpShooterStr<'pm> {
    participants: &'pm Vec<TeamMember>,
    my_pid: ParticipantId,
    // TODO: make a function of target AC ?
    use_ss: bool // dyn Fn(isize) -> bool,
}

impl<'pm> Strategy for SharpShooterStr<'pm> {
    fn get_participants(&self) -> &Vec<TeamMember> {
        self.participants
    }

    fn get_my_pid(&self) -> ParticipantId {
        self.my_pid
    }

    fn choose_action(&self, state: &CombatState) -> StrategyDecision {
        let me = self.get_my_pid();
        let my_rm = state.get_rm(me);
        if my_rm.get_current(ResourceName::RAT(ResourceActionType::Action)) > 0 {
            return ActionName::AttackAction.into()
        }
        if my_rm.get_current(ResourceName::RAT(ResourceActionType::SingleAttack)) > 0 {
            let target = self.get_first_target(state);
            let mut at = AttackType::Normal;
            if self.use_ss {
                at = AttackType::SSAttack;
            }
            return StrategicAction::targeted(ActionName::PrimaryAttack(at), target).into();
        }
        StrategyDecision::DoNothing
    }

    fn choose_triggers(&self, _: TriggerInfo, _: &CombatState) -> Vec<TriggerResponse> {
        Vec::new()
    }
}
