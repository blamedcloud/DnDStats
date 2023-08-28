use std::fmt::Debug;
use std::marker::PhantomData;

use rand_var::rv_traits::prob_type::RVProb;

use crate::actions::{ActionName, ActionType, AttackType};
use crate::attack::AttackResult;
use crate::combat_state::CombatState;
use crate::health::Health;
use crate::participant::{ParticipantId, TeamMember};
use crate::resources::ResourceName;
use crate::strategy::{StrategicOption, Strategy, StrategyBuilder};
use crate::triggers::{TriggerContext, TriggerName, TriggerResponse, TriggerType};

pub struct PairStrBuilder<T, S1, S2>
where
    T: RVProb,
    S1: StrategyBuilder<T>,
    S2: StrategyBuilder<T>,
{
    _t: PhantomData<T>,
    str1: S1,
    str2: S2,
}

impl<T, S1, S2> PairStrBuilder<T, S1, S2>
where
    T: RVProb,
    S1: StrategyBuilder<T>,
    S2: StrategyBuilder<T>,
{
    pub fn new(s1: S1, s2: S2) -> Self {
        Self {
            _t: PhantomData,
            str1: s1,
            str2: s2,
        }
    }
}

impl<T, S1, S2> StrategyBuilder<T> for PairStrBuilder<T, S1, S2>
where
    T: RVProb,
    S1: StrategyBuilder<T>,
    S2: StrategyBuilder<T>,
{
    fn build_strategy<'pm>(self, participants: &'pm Vec<TeamMember<T>>, me: ParticipantId) -> Box<dyn Strategy<T> + 'pm> {
        let strategies = vec!(
            self.str1.build_strategy(participants, me),
            self.str2.build_strategy(participants, me),
        );
        Box::new(LinearStrategy {
            strategies,
        })
    }
}

#[derive(Debug)]
pub struct LinearStrategy<'pm, T: RVProb> {
    strategies: Vec<Box<dyn Strategy<T> + 'pm>>,
}

impl<'pm, T: RVProb> Strategy<T> for LinearStrategy<'pm, T> {
    fn get_participants(&self) -> &Vec<TeamMember<T>> {
        self.strategies.first().unwrap().get_participants()
    }

    fn get_my_pid(&self) -> ParticipantId {
        self.strategies.first().unwrap().get_my_pid()
    }

    fn get_action(&self, state: &CombatState) -> Option<StrategicOption> {
        for str in self.strategies.iter() {
            let so = str.get_action(state);
            if so.is_some() {
                return so;
            }
        }
        None
    }

    fn handle_trigger(&self, tt: TriggerType, tc: TriggerContext, state: &CombatState) -> Vec<TriggerResponse> {
        let mut v = Vec::new();
        for str in self.strategies.iter() {
            let sub_v = str.handle_trigger(tt, tc, state);
            v.extend(sub_v.into_iter());
        }
        v
    }
}


pub struct DoNothingBuilder;
impl<T: RVProb> StrategyBuilder<T> for DoNothingBuilder {
    fn build_strategy<'pm>(self, _: &'pm Vec<TeamMember<T>>, _: ParticipantId) -> Box<dyn Strategy<T> + 'pm> {
        Box::new(DoNothing)
    }
}

#[derive(Debug)]
pub struct DoNothing;
impl<T: RVProb> Strategy<T> for DoNothing {
    fn get_participants(&self) -> &Vec<TeamMember<T>> {
        panic!("Should never call this!");
    }

    fn get_my_pid(&self) -> ParticipantId {
        panic!("Should never call this!");
    }

    fn get_action(&self, _: &CombatState) -> Option<StrategicOption> {
        None
    }

    fn handle_trigger(&self, _: TriggerType, _: TriggerContext, _: &CombatState) -> Vec<TriggerResponse> {
        Vec::new()
    }
}

pub struct BasicAtkStrBuilder;
impl<T: RVProb> StrategyBuilder<T> for BasicAtkStrBuilder {
    fn build_strategy<'pm>(self, participants: &'pm Vec<TeamMember<T>>, me: ParticipantId) -> Box<dyn Strategy<T> + 'pm> {
        let str = BasicAttackStr {
            participants,
            my_pid: me,
        };
        Box::new(str)
    }
}

#[derive(Debug)]
pub struct BasicAttackStr<'pm, T: RVProb> {
    participants: &'pm Vec<TeamMember<T>>,
    my_pid: ParticipantId,
}
impl<'pm, T: RVProb> Strategy<T> for BasicAttackStr<'pm, T> {
    fn get_participants(&self) -> &Vec<TeamMember<T>> {
        self.participants
    }

    fn get_my_pid(&self) -> ParticipantId {
        self.my_pid
    }

    fn get_action(&self, state: &CombatState) -> Option<StrategicOption> {
        let me = self.get_my_pid();
        let my_rm = state.get_rm(me);
        if my_rm.get_current(ResourceName::AT(ActionType::Action)) > 0 {
            return Some(ActionName::AttackAction.into())
        }
        if my_rm.get_current(ResourceName::AT(ActionType::SingleAttack)) > 0 {
            let target = self.get_first_target(state);
            return Some(StrategicOption {
                action_name: ActionName::PrimaryAttack(AttackType::Normal),
                target
            })
        }
        None
    }

    fn handle_trigger(&self, _: TriggerType, _: TriggerContext, _: &CombatState) -> Vec<TriggerResponse> {
        Vec::new()
    }
}


pub struct SecondWindStrBuilder;
impl<T: RVProb> StrategyBuilder<T> for SecondWindStrBuilder {
    fn build_strategy<'pm>(self, _: &'pm Vec<TeamMember<T>>, me: ParticipantId) -> Box<dyn Strategy<T> + 'pm> {
        let str = SecondWindStr {
            _t: PhantomData,
            my_pid: me,
        };
        Box::new(str)
    }
}

#[derive(Debug)]
pub struct SecondWindStr<T> {
    _t: PhantomData<T>,
    my_pid: ParticipantId
}
impl<T:RVProb> Strategy<T> for SecondWindStr<T> {
    fn get_participants(&self) -> &Vec<TeamMember<T>> {
        panic!("Should never call this!");
    }

    fn get_my_pid(&self) -> ParticipantId {
        self.my_pid
    }

    fn get_action(&self, state: &CombatState) -> Option<StrategicOption> {
        let me = self.get_my_pid();
        let my_rm = state.get_rm(me);
        let has_ba = my_rm.get_current(ResourceName::AT(ActionType::BonusAction)) > 0;
        let has_sw = my_rm.get_current(ResourceName::AN(ActionName::SecondWind)) > 0;
        if has_ba && has_sw && state.get_health(me) == Health::Bloodied {
            return Some(ActionName::SecondWind.into())
        }
        None
    }

    fn handle_trigger(&self, _: TriggerType, _: TriggerContext, _: &CombatState) -> Vec<TriggerResponse> {
        Vec::new()
    }
}


pub struct DualWieldStrBuilder;
impl<T: RVProb> StrategyBuilder<T> for DualWieldStrBuilder {
    fn build_strategy<'pm>(self, participants: &'pm Vec<TeamMember<T>>, me: ParticipantId) -> Box<dyn Strategy<T> + 'pm> {
        let str = DualWieldStr {
            participants,
            my_pid: me,
        };
        Box::new(str)
    }
}

#[derive(Debug)]
pub struct DualWieldStr<'pm, T: RVProb> {
    participants: &'pm Vec<TeamMember<T>>,
    my_pid: ParticipantId,
}
impl<'pm, T: RVProb> Strategy<T> for DualWieldStr<'pm, T> {
    fn get_participants(&self) -> &Vec<TeamMember<T>> {
        self.participants
    }

    fn get_my_pid(&self) -> ParticipantId {
        self.my_pid
    }

    fn get_action(&self, state: &CombatState) -> Option<StrategicOption> {
        let me = self.get_my_pid();
        let my_rm = state.get_rm(me);
        if my_rm.get_current(ResourceName::AT(ActionType::Action)) > 0 {
            return Some(ActionName::AttackAction.into());
        }
        if my_rm.get_current(ResourceName::AT(ActionType::SingleAttack)) > 0 {
            let target = self.get_first_target(state);
            return Some(StrategicOption {
                action_name: ActionName::PrimaryAttack(AttackType::Normal),
                target
            });
        }
        if my_rm.get_current(ResourceName::AT(ActionType::BonusAction)) > 0 {
            let target = self.get_first_target(state);
            return Some(StrategicOption {
                action_name: ActionName::OffhandAttack(AttackType::Normal),
                target
            });
        }
        None
    }

    fn handle_trigger(&self, _: TriggerType, _: TriggerContext, _: &CombatState) -> Vec<TriggerResponse> {
        Vec::new()
    }
}


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

    fn get_action(&self, _: &CombatState) -> Option<StrategicOption> {
        None
    }

    fn handle_trigger(&self, tt: TriggerType, tc: TriggerContext, state: &CombatState) -> Vec<TriggerResponse> {
        let mut v = Vec::new();
        if tt == TriggerType::SuccessfulAttack {
            let my_rm = state.get_rm(self.my_pid);
            let has_sa = my_rm.get_current(ResourceName::TN(TriggerName::SneakAttack)) > 0;
            if has_sa {
                let has_action = my_rm.get_current(ResourceName::AT(ActionType::Action)) > 0;
                let has_attacks = my_rm.get_current(ResourceName::AT(ActionType::SingleAttack)) > 0;
                let has_ba = my_rm.get_current(ResourceName::AT(ActionType::BonusAction)) > 0;
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
