use std::collections::HashSet;
use character_builder::Character;
use character_builder::combat::ActionManager;
use character_builder::damage::DamageType;

pub mod target_dummy;

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

pub struct EncounterManager {
    participants: Vec<TeamMember>, // in order of initiative
    turn_counter: u8,
}

impl EncounterManager {
    pub fn new() -> Self {
        EncounterManager {
            participants: Vec::new(),
            turn_counter: 0,
        }
    }

    pub fn add_participant(&mut self, tm: TeamMember) {
        self.participants.push(tm);
    }

    pub fn simulate_n_turns(&mut self, n: u8) {
        while self.turn_counter < n {
            self.simulate_turn();
        }
    }

    pub fn simulate_turn(&mut self) {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use character_builder::ability_scores::AbilityScores;
    use character_builder::Character;
    use character_builder::classes::ClassName;
    use character_builder::equipment::{Armor, Equipment, OffHand, Weapon};
    use character_builder::feature::fighting_style::{FightingStyle, FightingStyles};
    use crate::{EncounterManager, Team, TeamMember};
    use crate::target_dummy::TargetDummy;

    pub fn get_str_based() -> AbilityScores {
        AbilityScores::new(16,12,16,8,13,10)
    }

    pub fn get_test_fighter_lvl0() -> Character {
        let name = String::from("FighterMan");
        let ability_scores = get_str_based();
        let equipment = Equipment::new(
            Armor::chain_mail(),
            Weapon::greatsword(),
            OffHand::Free,
        );
        Character::new(name, ability_scores, equipment)
    }

    #[test]
    fn lvl1fighter_vs_dummy() {
        let mut fighter = get_test_fighter_lvl0();
        fighter.level_up(ClassName::Fighter, vec!(Box::new(FightingStyle(FightingStyles::GreatWeaponFighting)))).unwrap();
        let dummy = TargetDummy::new(isize::MAX, 14);

        let mut em = EncounterManager::new();
        em.add_participant(TeamMember::new(Team::Players, Box::new(fighter)));
        em.add_participant(TeamMember::new(Team::Enemies, Box::new(dummy)));
    }
}
