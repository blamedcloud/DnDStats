use crate::actions::{ActionName, AttackType};
use crate::combat_state::CombatState;
use crate::participant::{ParticipantId, TeamMember};
use crate::resources::{ResourceActionType, ResourceName};
use crate::strategy::{StrategicAction, Strategy, StrategyBuilder, StrategyDecision};
use crate::triggers::{TriggerContext, TriggerResponse, TriggerType};

pub struct PlanarWarriorStrBldr;
impl StrategyBuilder for PlanarWarriorStrBldr {
    fn build_strategy<'pm>(&self, participants: &'pm Vec<TeamMember>, me: ParticipantId) -> Box<dyn Strategy + 'pm> {
        let str = PlanarWarriorStr {
            participants,
            my_pid: me,
        };
        Box::new(str)
    }
}

#[derive(Debug)]
pub struct PlanarWarriorStr<'pm> {
    participants: &'pm Vec<TeamMember>,
    my_pid: ParticipantId,
}

impl<'pm> Strategy for PlanarWarriorStr<'pm> {
    fn get_participants(&self) -> &Vec<TeamMember> {
        self.participants
    }

    fn get_my_pid(&self) -> ParticipantId {
        self.my_pid
    }

    fn get_action(&self, state: &CombatState) -> StrategyDecision {
        let me = self.get_my_pid();
        let my_rm = state.get_rm(me);
        if my_rm.get_current(ResourceName::RAT(ResourceActionType::Action)) > 0 {
            return ActionName::AttackAction.into()
        }
        if my_rm.get_current(ResourceName::RAT(ResourceActionType::BonusAction)) > 0 {
            let target = self.get_first_target(state);
            return StrategicAction {
                action_name: ActionName::PlanarWarrior,
                target
            }.into()
        }
        if my_rm.get_current(ResourceName::RAT(ResourceActionType::SingleAttack)) > 0 {
            let target = self.get_first_target(state);
            return StrategicAction {
                action_name: ActionName::PrimaryAttack(AttackType::Normal),
                target
            }.into()
        }
        StrategyDecision::DoNothing
    }

    fn handle_trigger(&self, _: TriggerType, _: TriggerContext, _: &CombatState) -> Vec<TriggerResponse> {
        Vec::new()
    }
}
