use std::collections::HashSet;
use std::fmt::Debug;
use rand_var::rv_traits::prob_type::RVProb;
use crate::actions::ActionManager;
use crate::damage::DamageType;
use crate::resources::ResourceManager;
use crate::strategy::Strategy;

pub trait Participant<T: RVProb, E> : Debug {
    fn get_ac(&self) -> isize;
    fn get_max_hp(&self) -> isize;
    fn get_resistances(&self) -> &HashSet<DamageType>;
    fn get_action_manager(&self) -> &ActionManager<T, E>;
    fn get_resource_manager(&self) -> &ResourceManager;
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Team {
    Players,
    Enemies,
}

#[derive(Debug)]
pub struct TeamMember<T: RVProb, E> {
    pub team: Team,
    pub participant: Box<dyn Participant<T, E>>,
}
impl<T: RVProb, E> TeamMember<T, E> {
    pub fn new(team: Team, participant: Box<dyn Participant<T, E>>) -> Self {
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
pub struct ParticipantManager<T: RVProb, E> { // In order of initiative
    participants: Vec<TeamMember<T, E>>,
    initial_resources: Vec<ResourceManager>,
    strategies: Vec<Box<dyn Strategy<T, E>>>,
}

impl<T: RVProb, E> ParticipantManager<T, E> {
    pub fn new() -> Self {
        Self {
            participants: Vec::new(),
            initial_resources: Vec::new(),
            strategies: Vec::new(),
        }
    }

    pub fn add_player(&mut self, player: Box<dyn Participant<T, E>>, str: Box<dyn Strategy<T, E>>) {
        self.add_participant(TeamMember::new(Team::Players, player), str);
    }

    pub fn add_enemy(&mut self, enemy: Box<dyn Participant<T, E>>, str: Box<dyn Strategy<T, E>>) {
        self.add_participant(TeamMember::new(Team::Enemies, enemy), str);
    }

    pub fn add_participant(&mut self, tm: TeamMember<T, E>, str: Box<dyn Strategy<T, E>>) {
        self.initial_resources.push(tm.participant.get_resource_manager().clone());
        self.participants.push(tm);
        self.strategies.push(str);
    }

    pub fn len(&self) -> usize {
        self.participants.len()
    }

    pub fn get_participants(&self) -> &Vec<TeamMember<T, E>> {
        &self.participants
    }

    pub fn get_participant(&self, pid: ParticipantId) -> &TeamMember<T, E> {
        self.participants.get(pid.0).unwrap()
    }

    pub fn get_strategy(&self, pid: ParticipantId) -> &Box<dyn Strategy<T, E>> {
        self.strategies.get(pid.0).unwrap()
    }

    pub fn get_initial_rms(&self) -> Vec<ResourceManager> {
        self.initial_resources.clone()
    }
}
