use crate::actions::{ActionName, AttackType};
use crate::attack::AttackResult;
use crate::combat_state::CombatState;
use crate::participant::{ParticipantId, TeamMember};
use crate::resources::{ResourceActionType, ResourceName};
use crate::strategy::{StrategicAction, Strategy, StrategyBuilder, StrategyDecision};
use crate::triggers::{TriggerContext, TriggerInfo, TriggerName, TriggerResponse, TriggerType};

pub struct GWMStrBldr {
    use_gwm: bool // dyn Fn(isize) -> bool,
}
impl GWMStrBldr {
    pub fn new(use_gwm: bool) -> Self {
        Self {
            use_gwm,
        }
    }
}
impl StrategyBuilder for GWMStrBldr {
    fn build_strategy<'pm>(&self, participants: &'pm Vec<TeamMember>, me: ParticipantId) -> Box<dyn Strategy + 'pm> {
        let str = GWMStr {
            participants,
            my_pid: me,
            use_gwm: self.use_gwm,
        };
        Box::new(str)
    }
}

#[derive(Debug)]
pub struct GWMStr<'pm> {
    participants: &'pm Vec<TeamMember>,
    my_pid: ParticipantId,
    // TODO: make a function of target AC ?
    use_gwm: bool // dyn Fn(isize) -> bool,
}

impl<'pm> Strategy for GWMStr<'pm> {
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
        if my_rm.get_current(ResourceName::AN(ActionName::BonusGWMAttack)) > 0 && my_rm.get_current(ResourceName::RAT(ResourceActionType::BonusAction)) > 0 {
            return ActionName::BonusGWMAttack.into()
        }
        if my_rm.get_current(ResourceName::RAT(ResourceActionType::SingleAttack)) > 0 {
            let target = self.get_first_target(state);
            let mut at = AttackType::Normal;
            if self.use_gwm {
                at = AttackType::GWMAttack;
            }
            return StrategicAction {
                action_name: ActionName::PrimaryAttack(at),
                target
            }.into()
        }
        StrategyDecision::DoNothing
    }

    fn handle_trigger(&self, ti: TriggerInfo, _: &CombatState) -> Vec<TriggerResponse> {
        let mut v = Vec::new();
        let on_crit = TriggerInfo::new(TriggerType::SuccessfulAttack, TriggerContext::AR(AttackResult::Crit));
        let on_kill = TriggerInfo::new(TriggerType::OnKill, TriggerContext::NoContext);
        if ti == on_crit || ti == on_kill {
            let my_tm = self.get_me().get_trigger_manager().unwrap();
            v.push(my_tm.get_response(TriggerName::GWMBonusAtk).unwrap());
        }
        v
    }
}
