use crate::actions::ActionName;
use crate::combat_state::CombatState;
use crate::conditions::{ConditionLifetime, ConditionName};
use crate::participant::{ParticipantId, TeamMember};
use crate::resources::{ResourceActionType, ResourceName};
use crate::strategy::{StrategicAction, Strategy, StrategyBuilder, StrategyDecision};
use crate::triggers::{TriggerInfo, TriggerResponse};

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

    fn choose_action(&self, state: &CombatState) -> StrategyDecision {
        let me = self.get_my_pid();
        let my_rm = state.get_rm(me);
        if my_rm.get_current(ResourceName::RAT(ResourceActionType::BonusAction)) > 0 {
            let ffa = ResourceName::AN(ActionName::FavoredFoeApply);
            if my_rm.get_current(ffa) > 0 {
                let target = self.get_first_target(state);
                return StrategicAction::targeted(ActionName::FavoredFoeApply, target).into();
            }
            let ffu = ResourceName::AN(ActionName::FavoredFoeUse);
            let cast_action_spell = state.get_cm(me).has_condition(&ConditionName::CastActionSpell);
            if !cast_action_spell && my_rm.get_current(ffu) > 0 && self.use_ff(state) {
                let target = self.get_first_target(state);
                return StrategicAction::targeted(ActionName::FavoredFoeUse, target).into();
            }
        }
        StrategyDecision::DoNothing
    }

    fn choose_triggers(&self, _: TriggerInfo, _: &CombatState) -> Vec<TriggerResponse> {
        Vec::new()
    }
}
