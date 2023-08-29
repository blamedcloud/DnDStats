use rand_var::rv_traits::prob_type::RVProb;

use crate::actions::{ActionName, AttackType};
use crate::attack::AttackResult;
use crate::combat_state::CombatState;
use crate::participant::{ParticipantId, TeamMember};
use crate::resources::{ResourceActionType, ResourceName};
use crate::strategy::{Strategy, StrategyBuilder, StrategyDecision};
use crate::triggers::{TriggerContext, TriggerName, TriggerResponse, TriggerType};

pub struct SneakAttackStrBuilder {
    greedy: bool
}

impl SneakAttackStrBuilder {
    pub fn new(greedy: bool) -> Self {
        Self {
            greedy
        }
    }
}

impl<T: RVProb> StrategyBuilder<T> for SneakAttackStrBuilder {
    fn build_strategy<'pm>(self, participants: &'pm Vec<TeamMember<T>>, me: ParticipantId) -> Box<dyn Strategy<T> + 'pm> {
        let str = SneakAttackStr {
            participants,
            my_pid: me,
            greedy: self.greedy
        };
        Box::new(str)
    }
}

#[derive(Debug)]
pub struct SneakAttackStr<'pm, T: RVProb> {
    participants: &'pm Vec<TeamMember<T>>,
    my_pid: ParticipantId,
    greedy: bool,
}

impl<'pm, T: RVProb> Strategy<T> for SneakAttackStr<'pm, T> {
    fn get_participants(&self) -> &Vec<TeamMember<T>> {
        self.participants
    }

    fn get_my_pid(&self) -> ParticipantId {
        self.my_pid
    }

    fn get_action(&self, _: &CombatState) -> StrategyDecision {
        StrategyDecision::DoNothing
    }

    fn handle_trigger(&self, tt: TriggerType, tc: TriggerContext, state: &CombatState) -> Vec<TriggerResponse> {
        let mut v = Vec::new();
        if tt == TriggerType::SuccessfulAttack {
            let my_rm = state.get_rm(self.my_pid);
            let has_sa = my_rm.get_current(ResourceName::TN(TriggerName::SneakAttack)) > 0;
            if has_sa {
                let has_action = my_rm.get_current(ResourceName::RAT(ResourceActionType::Action)) > 0;
                let has_attacks = my_rm.get_current(ResourceName::RAT(ResourceActionType::SingleAttack)) > 0;
                let has_ba = my_rm.get_current(ResourceName::RAT(ResourceActionType::BonusAction)) > 0;
                let has_ofa = self.get_me().get_action_manager().contains_key(&ActionName::OffhandAttack(AttackType::Normal));
                let any_atk_remaining = has_action || has_attacks || (has_ba && has_ofa);
                if self.greedy && any_atk_remaining {
                    if let TriggerContext::AR(AttackResult::Crit) = tc {
                        let my_tm = self.get_me().get_trigger_manager().unwrap();
                        v.push(my_tm.get_response(TriggerName::SneakAttack).unwrap());
                    }
                } else {
                    let my_tm = self.get_me().get_trigger_manager().unwrap();
                    v.push(my_tm.get_response(TriggerName::SneakAttack).unwrap());
                }
            }
        }
        v
    }
}
