use std::fmt::Debug;
use std::marker::PhantomData;
use rand_var::rv_traits::prob_type::RVProb;
use crate::actions::{ActionName, ActionType, AttackType};
use crate::combat_state::CombatState;
use crate::health::Health;
use crate::participant::{ParticipantId, TeamMember};
use crate::resources::ResourceName;
use crate::strategy::{StrategicOption, Strategy, StrategyBuilder, Target};
use crate::triggers::{TriggeredAction, TriggerType};

pub struct PairStrBuilder<T: RVProb, S1: StrategyBuilder<T>, S2: StrategyBuilder<T>> {
    _t: PhantomData<T>,
    str1: S1,
    str2: S2,
}

impl <T: RVProb, S1: StrategyBuilder<T>, S2: StrategyBuilder<T>> PairStrBuilder<T, S1, S2> {
    pub fn new(s1: S1, s2: S2) -> Self {
        Self {
            _t: PhantomData,
            str1: s1,
            str2: s2,
        }
    }
}

impl<T: RVProb, S1: StrategyBuilder<T>, S2: StrategyBuilder<T>> StrategyBuilder<T> for PairStrBuilder<T, S1, S2> {
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

    fn get_me(&self) -> ParticipantId {
        self.strategies.first().unwrap().get_me()
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

    fn handle_trigger(&self, tt: TriggerType, state: &CombatState) -> Vec<TriggeredAction> {
        let mut v = Vec::new();
        for str in self.strategies.iter() {
            let sub_v = str.handle_trigger(tt, state);
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

    fn get_me(&self) -> ParticipantId {
        panic!("Should never call this!");
    }

    fn get_action(&self, _: &CombatState) -> Option<StrategicOption> {
        None
    }

    fn handle_trigger(&self, _: TriggerType, _: &CombatState) -> Vec<TriggeredAction> {
        Vec::new()
    }
}

pub fn get_first_target<T: RVProb>(state: &CombatState, participants: &Vec<TeamMember<T>>, me: ParticipantId) -> Option<Target> {
    let my_team = participants.get(me.0).unwrap().team;
    for i in 0..participants.len() {
        let pid = ParticipantId(i);
        if participants[i].team != my_team && state.is_alive(pid) {
            return Some(Target::Participant(pid))
        }
    }
    None
}

pub struct BasicAtkStrBuilder;
impl<T: RVProb> StrategyBuilder<T> for BasicAtkStrBuilder {
    fn build_strategy<'pm>(self, participants: &'pm Vec<TeamMember<T>>, me: ParticipantId) -> Box<dyn Strategy<T> + 'pm> {
        let str = BasicAttackStr {
            participants,
            me,
        };
        Box::new(str)
    }
}

#[derive(Debug)]
pub struct BasicAttackStr<'pm, T: RVProb> {
    participants: &'pm Vec<TeamMember<T>>,
    me: ParticipantId,
}

impl<'pm, T: RVProb> Strategy<T> for BasicAttackStr<'pm, T> {
    fn get_participants(&self) -> &Vec<TeamMember<T>> {
        self.participants
    }

    fn get_me(&self) -> ParticipantId {
        self.me
    }

    fn get_action(&self, state: &CombatState) -> Option<StrategicOption> {
        let participants = self.get_participants();
        let me = self.get_me();
        let my_rm = state.get_rm(me);
        if my_rm.get_current(ResourceName::AT(ActionType::Action)) > 0 {
            return Some(ActionName::AttackAction.into())
        }
        if my_rm.get_current(ResourceName::AT(ActionType::SingleAttack)) > 0 {
            let target = get_first_target(state, participants, me);
            return Some(StrategicOption {
                action_name: ActionName::PrimaryAttack(AttackType::Normal),
                target
            })
        }
        None
    }

    fn handle_trigger(&self, _: TriggerType, _: &CombatState) -> Vec<TriggeredAction> {
        Vec::new()
    }
}

pub struct SecondWindStrBuilder;
impl<T: RVProb> StrategyBuilder<T> for SecondWindStrBuilder {
    fn build_strategy<'pm>(self, _: &'pm Vec<TeamMember<T>>, me: ParticipantId) -> Box<dyn Strategy<T> + 'pm> {
        let str = SecondWindStr {
            _t: PhantomData,
            me,
        };
        Box::new(str)
    }
}

#[derive(Debug)]
pub struct SecondWindStr<T> {
    _t: PhantomData<T>,
    me: ParticipantId
}
impl<T:RVProb> Strategy<T> for SecondWindStr<T> {
    fn get_participants(&self) -> &Vec<TeamMember<T>> {
        panic!("Should never call this!");
    }

    fn get_me(&self) -> ParticipantId {
        self.me
    }

    fn get_action(&self, state: &CombatState) -> Option<StrategicOption> {
        let me = self.get_me();
        let my_rm = state.get_rm(me);
        let has_ba = my_rm.get_current(ResourceName::AT(ActionType::BonusAction)) > 0;
        let has_sw = my_rm.get_current(ResourceName::AN(ActionName::SecondWind)) > 0;
        if has_ba && has_sw && state.get_health(me) == Health::Bloodied {
            return Some(ActionName::SecondWind.into())
        }
        None
    }

    fn handle_trigger(&self, _: TriggerType, _: &CombatState) -> Vec<TriggeredAction> {
        Vec::new()
    }
}
