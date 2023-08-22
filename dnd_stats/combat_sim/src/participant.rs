use std::collections::HashSet;
use character_builder::Character;
use character_builder::combat::ActionManager;
use character_builder::damage::DamageType;

pub trait Participant {
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

#[derive(Debug, Ord, PartialOrd, PartialEq, Eq, Clone, Copy)]
pub struct ParticipantId(pub usize);
impl From<usize> for ParticipantId {
    fn from(value: usize) -> Self {
        ParticipantId(value)
    }
}
