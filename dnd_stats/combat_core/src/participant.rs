use std::collections::HashSet;
use std::fmt::Debug;
use rand_var::rv_traits::prob_type::RVProb;
use crate::actions::ActionManager;
use crate::CCError;
use crate::damage::DamageType;
use crate::resources::ResourceManager;

pub trait Participant<T: RVProb> : Debug {
    fn get_ac(&self) -> isize;
    fn get_max_hp(&self) -> isize;
    fn get_resistances(&self) -> &HashSet<DamageType>;
    fn get_action_manager(&self) -> &ActionManager<T>;
    fn get_resource_manager(&self) -> &ResourceManager;
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Team {
    Players,
    Enemies,
}

#[derive(Debug)]
pub struct TeamMember<T: RVProb> {
    pub team: Team,
    pub participant: Box<dyn Participant<T>>,
}
impl<T: RVProb> TeamMember<T> {
    pub fn new(team: Team, participant: Box<dyn Participant<T>>) -> Self {
        TeamMember {
            team,
            participant,
        }
    }
}

#[derive(Debug, Ord, PartialOrd, PartialEq, Eq, Clone, Copy, Hash)]
pub struct ParticipantId(pub usize);
impl From<usize> for ParticipantId {
    fn from(value: usize) -> Self {
        ParticipantId(value)
    }
}

#[derive(Debug)]
pub struct ParticipantManager<T: RVProb> { // In order of initiative
    participants: Vec<TeamMember<T>>,
    initial_resources: Vec<ResourceManager>,
    compiled: bool,
}

impl<T: RVProb> ParticipantManager<T> {
    pub fn new() -> Self {
        Self {
            participants: Vec::new(),
            initial_resources: Vec::new(),
            compiled: false,
        }
    }

    pub fn add_player(&mut self, player: Box<dyn Participant<T>>) -> Result<(), CCError> {
        self.add_participant(TeamMember::new(Team::Players, player))
    }

    pub fn add_enemy(&mut self, enemy: Box<dyn Participant<T>>) -> Result<(), CCError> {
        self.add_participant(TeamMember::new(Team::Enemies, enemy))
    }

    pub fn add_participant(&mut self, tm: TeamMember<T>) -> Result<(), CCError> {
        if self.compiled {
            return Err(CCError::PMPushAfterCompile);
        }
        self.initial_resources.push(tm.participant.get_resource_manager().clone());
        self.participants.push(tm);
        Ok(())
    }

    pub fn compile(&mut self) {
        self.compiled = true;
    }

    pub fn is_compiled(&self) -> bool {
        self.compiled
    }

    pub fn len(&self) -> usize {
        self.participants.len()
    }

    pub fn get_participants(&self) -> &Vec<TeamMember<T>> {
        &self.participants
    }

    pub fn get_participant(&self, pid: ParticipantId) -> &TeamMember<T> {
        self.participants.get(pid.0).unwrap()
    }

    pub fn get_initial_rms(&self) -> Vec<ResourceManager> {
        self.initial_resources.clone()
    }
}
