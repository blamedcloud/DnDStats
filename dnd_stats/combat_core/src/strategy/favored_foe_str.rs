use crate::actions::{ActionName, AttackType};
use crate::combat_state::CombatState;
use crate::conditions::{ConditionLifetime, ConditionName};
use crate::participant::{ParticipantId, TeamMember};
use crate::resources::{ResourceActionType, ResourceName};
use crate::strategy::{StrategicAction, Strategy, StrategyBuilder, StrategyDecision};
use crate::triggers::{TriggerContext, TriggerInfo, TriggerName, TriggerResponse, TriggerType};

pub struct FavoredFoeStrBldr;
impl StrategyBuilder for FavoredFoeStrBldr {
    fn build_strategy<'pm>(&self, participants: &'pm Vec<TeamMember>, me: ParticipantId) -> Box<dyn Strategy + 'pm> {
        let str = FavoredFoeStr {
            participants,
            my_pid: me,
        };
        Box::new(str)
    }
}

#[derive(Debug)]
pub struct FavoredFoeStr<'pm> {
    participants: &'pm Vec<TeamMember>,
    my_pid: ParticipantId,
}
impl<'pm> FavoredFoeStr<'pm> {
    fn use_ff(&self, state: &CombatState) -> bool {
        let participants = self.get_participants();
        let me = self.get_my_pid();
        let my_team = participants.get(me.0).unwrap().team;
        for i in 0..participants.len() {
            let pid = ParticipantId(i);
            if participants[i].team != my_team  && state.is_alive(pid) {
                let cm = state.get_cm(pid);
                if cm.has_condition_with_lifetime(&ConditionLifetime::NotifyOnDeath(me), &ConditionName::FavoredFoe) {
                    return false;
                }
            }
        }
        true
    }
}

impl<'pm> Strategy for FavoredFoeStr<'pm> {
    fn get_participants(&self) -> &Vec<TeamMember> {
        self.participants
    }

    fn get_my_pid(&self) -> ParticipantId {
        self.my_pid
    }

    fn get_action(&self, state: &CombatState) -> StrategyDecision {
        let me = self.get_my_pid();
        let my_rm = state.get_rm(me);
        if my_rm.get_current(ResourceName::RAT(ResourceActionType::BonusAction)) > 0 {
            let ffa = ResourceName::AN(ActionName::FavoredFoeApply);
            if my_rm.get_current(ffa) > 0 {
                let target = self.get_first_target(state);
                return StrategicAction {
                    action_name: ActionName::FavoredFoeApply,
                    target
                }.into();
            }
            if my_rm.get_current(ResourceName::AN(ActionName::FavoredFoeUse)) > 0 && self.use_ff(state) {
                return ActionName::FavoredFoeUse.into()
            }
        }

        if my_rm.get_current(ResourceName::RAT(ResourceActionType::Action)) > 0 {
            return ActionName::AttackAction.into()
        }
        if my_rm.get_current(ResourceName::RAT(ResourceActionType::SingleAttack)) > 0 {
            let target = self.get_first_target(state);
            return StrategicAction {
                action_name: ActionName::PrimaryAttack(AttackType::Normal),
                target
            }.into();
        }
        StrategyDecision::DoNothing
    }

    fn handle_trigger(&self, ti: TriggerInfo, _: &CombatState) -> Vec<TriggerResponse> {
        let mut v = Vec::new();
        if ti.tt == TriggerType::OnKill && ti.tc == TriggerContext::CondNotice(ConditionName::FavoredFoe) {
            let my_tm = self.get_me().get_trigger_manager().unwrap();
            v.push(my_tm.get_response(TriggerName::FavoredFoeKill).unwrap());
        }
        v
    }
}
