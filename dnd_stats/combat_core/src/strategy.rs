use std::fmt::Debug;

use crate::actions::{ActionName, ActionType};
use crate::CCError;
use crate::combat_state::CombatState;
use crate::conditions::ConditionName;
use crate::movement::Square;
use crate::participant::{Participant, ParticipantId, ParticipantManager, TeamMember};
use crate::spells::SpellSlot;
use crate::triggers::{TriggerInfo, TriggerResponse};

// I don't really like having so many strategy modules
// So maybe I should think about creating a more generally
// applicable one? It's much easier to make sure everything
// is working properly with them individual though.

pub mod action_surge_str;
pub mod basic_atk_str;
pub mod basic_strategies;
pub mod dual_wield_str;
pub mod favored_foe_str;
pub mod fireball_str;
pub mod firebolt_str;
pub mod greater_invis_str;
pub mod gwm_str;
pub mod haste_str;
pub mod linear_str;
pub mod planar_warrior_str;
pub mod second_wind_str;
pub mod sharp_shooter_str;
pub mod shield_master_str;
pub mod sneak_atk_str;

pub mod serialization;

// TODO: add shapes eventually (for spells and such)
#[derive(Debug, Ord, PartialOrd, PartialEq, Eq, Clone, Copy)]
pub enum Target {
    Participant(ParticipantId),
    Tile(Square, Square),
}

#[derive(Debug, Ord, PartialOrd, PartialEq, Eq, Clone, Copy)]
pub enum StrategyDecision {
    DoNothing,
    MyAction(StrategicAction),
    // done like this because the action manager
    // can't be modified, and probably doesn't
    // know it can spend an action to do this.
    RemoveCondition(ConditionName, ActionType),
}

impl StrategyDecision {
    pub fn is_some(&self) -> bool {
        !self.is_none()
    }

    pub fn is_none(&self) -> bool {
        self == &StrategyDecision::DoNothing
    }
}

impl From<StrategicAction> for StrategyDecision {
    fn from(value: StrategicAction) -> Self {
        StrategyDecision::MyAction(value)
    }
}
impl From<Option<StrategicAction>> for StrategyDecision {
    fn from(value: Option<StrategicAction>) -> Self {
        if value.is_some() {
            StrategyDecision::MyAction(value.unwrap())
        } else {
            StrategyDecision::DoNothing
        }
    }
}
impl From<ActionName> for StrategyDecision {
    fn from(value: ActionName) -> Self {
        StrategyDecision::MyAction(value.into())
    }
}

#[derive(Debug, Ord, PartialOrd, PartialEq, Eq, Clone, Copy)]
pub struct StrategicAction {
    pub action_name: ActionName,
    pub target: Option<Target>,
    pub spell_slot: Option<SpellSlot>,
}

impl StrategicAction {
    pub fn new(an: ActionName, target: Option<Target>, ss: Option<SpellSlot>) -> Self {
        Self {
            action_name: an,
            target,
            spell_slot: ss,
        }
    }

    pub fn targeted(an: ActionName, target: Option<Target>) -> Self {
        Self {
            action_name: an,
            target,
            spell_slot: None,
        }
    }

    pub fn spell(an: ActionName, ss: Option<SpellSlot>) -> Self {
        Self {
            action_name: an,
            target: None,
            spell_slot: ss,
        }
    }
}

impl From<ActionName> for StrategicAction {
    fn from(value: ActionName) -> Self {
        Self {
            action_name: value,
            target: None,
            spell_slot: None,
        }
    }
}

pub trait StrategyBuilder {
    fn build_strategy<'pm>(&self, participants: &'pm Vec<TeamMember>, me: ParticipantId) -> Box<dyn Strategy + 'pm>;
}

pub trait Strategy : Debug {
    fn get_participants(&self) -> &Vec<TeamMember>;
    fn get_my_pid(&self) -> ParticipantId;

    fn choose_action(&self, state: &CombatState) -> StrategyDecision;

    fn choose_triggers(&self, ti: TriggerInfo, state: &CombatState) -> Vec<TriggerResponse>;

    fn get_me(&self) -> &Box<dyn Participant> {
        &self.get_participants().get(self.get_my_pid().0).unwrap().participant
    }

    fn get_first_target(&self, state: &CombatState) -> Option<Target> {
        let participants = self.get_participants();
        let me = self.get_my_pid();
        let my_team = participants.get(me.0).unwrap().team;
        for i in 0..participants.len() {
            let pid = ParticipantId(i);
            if participants[i].team != my_team && state.is_alive(pid) {
                return Some(Target::Participant(pid))
            }
        }
        None
    }
}

pub struct StrategyManager<'pm> {
    pm: &'pm ParticipantManager,
    strategies: Vec<Box<dyn Strategy + 'pm>>,
    compiled: bool
}

impl<'pm> StrategyManager<'pm> {
    pub fn new(pm: &'pm ParticipantManager) -> Result<Self, CCError> {
        if !pm.is_compiled() {
            return Err(CCError::PMNotCompiled);
        }
        Ok(Self {
            pm,
            strategies: Vec::new(),
            compiled: false,
        })
    }

    pub fn add_participant(&mut self, str_bldr: impl StrategyBuilder) -> Result<(), CCError> {
        if self.compiled {
            return Err(CCError::SMPushAfterCompile);
        }
        let str = str_bldr.build_strategy(self.pm.get_participants(), ParticipantId(self.strategies.len()));
        self.strategies.push(str);

        if self.len() == self.pm.len() {
            self.compile();
        }

        Ok(())
    }

    pub fn get_pm(&self) -> &'pm ParticipantManager {
        self.pm
    }

    pub fn is_compiled(&self) -> bool {
        self.compiled
    }

    fn compile(&mut self) {
        self.compiled = true;
    }

    pub fn len(&self) -> usize {
        self.strategies.len()
    }

    pub fn get_strategy(&self, pid: ParticipantId) -> &Box<dyn Strategy + 'pm> {
        self.strategies.get(pid.0).unwrap()
    }
}
