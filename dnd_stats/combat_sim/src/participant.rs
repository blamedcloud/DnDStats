use std::collections::HashSet;
use std::fmt::Debug;
use character_builder::Character;
use character_builder::combat::ActionManager;
use character_builder::damage::DamageType;
use character_builder::resources::ResourceManager;
use crate::strategy::Strategy;

pub trait Participant : Debug {
    fn get_ac(&self) -> isize;
    fn get_max_hp(&self) -> isize;
    fn get_resistances(&self) -> &HashSet<DamageType>;
    fn get_action_manager(&self) -> &ActionManager;
}

impl Participant for Character {
    fn get_ac(&self) -> isize {
        self.get_ac() as isize
    }

    fn get_max_hp(&self) -> isize {
        self.get_max_hp()
    }

    fn get_resistances(&self) -> &HashSet<DamageType> {
        self.get_resistances()
    }

    fn get_action_manager(&self) -> &ActionManager {
        self.get_action_manager()
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Team {
    Players,
    Enemies,
}

#[derive(Debug)]
pub struct TeamMember {
    pub team: Team,
    pub participant: Box<dyn Participant>,
}
impl TeamMember {
    pub fn new(team: Team, participant: Box<dyn Participant>) -> Self {
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
pub struct ParticipantManager { // In order of initiative
    participants: Vec<TeamMember>,
    initial_resources: Vec<ResourceManager>,
    strategies: Vec<Box<dyn Strategy>>,
}

impl ParticipantManager {
    pub fn new() -> Self {
        Self {
            participants: Vec::new(),
            initial_resources: Vec::new(),
            strategies: Vec::new(),
        }
    }

    pub fn add_player(&mut self, character: Character, str: Box<dyn Strategy>) {
        let rm = character.get_resource_manager().clone();
        self.add_participant(TeamMember::new(Team::Players, Box::new(character)), rm, str);
    }

    pub fn add_participant(&mut self, tm: TeamMember, rm: ResourceManager, str: Box<dyn Strategy>) {
        self.participants.push(tm);
        self.initial_resources.push(rm);
        self.strategies.push(str);
    }

    pub fn len(&self) -> usize {
        self.participants.len()
    }

    pub fn get_participants(&self) -> &Vec<TeamMember> {
        &self.participants
    }

    pub fn get_participant(&self, pid: ParticipantId) -> &TeamMember {
        self.participants.get(pid.0).unwrap()
    }

    pub fn get_strategy(&self, pid: ParticipantId) -> &Box<dyn Strategy> {
        self.strategies.get(pid.0).unwrap()
    }

    pub fn get_initial_rms(&self) -> Vec<ResourceManager> {
        self.initial_resources.clone()
    }
}
