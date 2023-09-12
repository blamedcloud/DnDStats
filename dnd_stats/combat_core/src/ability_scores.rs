use std::fmt::{Display, Formatter};
use rand_var::map_rand_var::MapRandVar;

use rand_var::vec_rand_var::VecRandVar;
use rand_var::num_rand_var::NumRandVar;
use rand_var::rand_var::prob_type::RVProb;

use crate::{BinaryOutcome, D20RollType, D20Type};
use crate::combat_event::CombatEvent;
use crate::participant::Participant;

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone)]
pub enum Ability {
    STR,
    DEX,
    CON,
    INT,
    WIS,
    CHA,
}

impl Display for Ability {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut s = String::new();
        match self {
            Ability::STR => s.push_str("STR"),
            Ability::DEX => s.push_str("DEX"),
            Ability::CON => s.push_str("CON"),
            Ability::INT => s.push_str("INT"),
            Ability::WIS => s.push_str("WIS"),
            Ability::CHA => s.push_str("CHA"),
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct AbilityScore {
    score: u8,
    cap: u8,
    prof_save: bool,
    save_bonus: i8,
    default_roll_type: D20RollType,
    d20_type: D20Type,
}

impl AbilityScore {
    pub fn new(score: u8) -> Self {
        Self {
            score,
            cap: 20,
            prof_save: false,
            save_bonus: 0,
            default_roll_type: D20RollType::Normal,
            d20_type: D20Type::D20,
        }
    }

    pub fn get_score(&self) -> u8 {
        self.score
    }

    pub fn set_score(&mut self, new_score: u8) {
        if new_score <= self.cap {
            self.score = new_score;
        } else {
            self.score = self.cap;
        }
    }

    pub fn get_cap(&self) -> u8 {
        self.cap
    }

    pub fn get_mod(&self) -> i8 {
        ((self.score/2) as i8) - 5
    }

    pub fn increase(&mut self) {
        if self.score < self.cap {
            self.score += 1;
        }
    }

    pub fn increase_cap(&mut self) {
        self.cap += 1;
    }

    pub fn is_prof_save(&self) -> bool {
        self.prof_save
    }

    pub fn set_prof_save(&mut self, prof: bool) {
        self.prof_save = prof;
    }

    pub fn get_save_bonus(&self) -> i8 {
        self.save_bonus
    }

    pub fn set_save_bonus(&mut self, bonus: i8) {
        self.save_bonus = bonus;
    }

    pub fn add_save_bonus(&mut self, bonus: i8) {
        self.save_bonus += bonus;
    }

    pub fn get_d20_type(&self) -> &D20Type {
        &self.d20_type
    }

    pub fn get_d20_roll_type(&self) -> &D20RollType {
        &self.default_roll_type
    }

    pub fn get_save_rv<P: RVProb>(&self, prof: isize) -> VecRandVar<P> {
        let mut bonus = (self.get_mod() + self.save_bonus) as isize;
        if self.prof_save {
            bonus += prof;
        }
        let d20 = self.default_roll_type.get_rv(&self.d20_type);
        d20.add_const(bonus)
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct AbilityScores {
    pub strength: AbilityScore,
    pub dexterity: AbilityScore,
    pub constitution: AbilityScore,
    pub intelligence: AbilityScore,
    pub wisdom: AbilityScore,
    pub charisma: AbilityScore,
}

impl AbilityScores {
    pub fn new(str: u8, dex: u8, con: u8, int: u8, wis: u8, cha: u8) -> AbilityScores {
        AbilityScores {
            strength: AbilityScore::new(str),
            dexterity: AbilityScore::new(dex),
            constitution: AbilityScore::new(con),
            intelligence: AbilityScore::new(int),
            wisdom: AbilityScore::new(wis),
            charisma: AbilityScore::new(cha),
        }
    }

    pub fn get_score(&self, ability: &Ability) -> &AbilityScore {
        let score = match ability {
            &Ability::STR => &self.strength,
            &Ability::DEX => &self.dexterity,
            &Ability::CON => &self.constitution,
            &Ability::INT => &self.intelligence,
            &Ability::WIS => &self.wisdom,
            &Ability::CHA => &self.charisma,
        };
        score
    }

    pub fn get_score_mut(&mut self, ability: &Ability) -> &mut AbilityScore {
        let score = match ability {
            &Ability::STR => &mut self.strength,
            &Ability::DEX => &mut self.dexterity,
            &Ability::CON => &mut self.constitution,
            &Ability::INT => &mut self.intelligence,
            &Ability::WIS => &mut self.wisdom,
            &Ability::CHA => &mut self.charisma,
        };
        score
    }

    pub fn save_vs_dc<P: RVProb>(&self, ability: &Ability, prof: isize, dc: isize) -> MapRandVar<BinaryOutcome, P> {
        let save_rv = self.get_score(ability).get_save_rv(prof).into_mrv();
        save_rv.map_keys(|total| {
            if total >= dc {
                BinaryOutcome::Pass
            } else {
                BinaryOutcome::Fail
            }
        })
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct ForceSave {
    pub ability: Ability,
    pub save_dc: isize,
}

impl ForceSave {
    pub fn new(ability: Ability, save_dc: isize) -> Self {
        Self {
            ability,
            save_dc,
        }
    }

    pub fn make_save<P: RVProb>(&self, target: &dyn Participant) -> MapRandVar<CombatEvent, P> {
        let rv = self.get_save_rv(target.get_ability_scores(), target.get_prof());
        rv.map_keys(|sr| CombatEvent::SaveResult(sr))
    }

    pub fn get_save_rv<P: RVProb>(&self, ability_scores: &AbilityScores, prof: isize) -> MapRandVar<BinaryOutcome, P> {
        ability_scores.save_vs_dc(&self.ability, prof, self.save_dc)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_ability_test() {
        let ability = AbilityScore::new(1);
        assert_eq!(ability.get_mod(), -5);
        let ability = AbilityScore::new(10);
        assert_eq!(ability.get_mod(), 0);
        let ability = AbilityScore::new(13);
        assert_eq!(ability.get_mod(), 1);
        let mut ability = AbilityScore::new(16);
        assert_eq!(ability.get_mod(), 3);
        ability.increase(); // 17
        assert_eq!(ability.get_mod(), 3);
        ability.increase(); // 18
        assert_eq!(ability.get_mod(), 4);
        ability = AbilityScore::new(20);
        assert_eq!(ability.get_mod(), 5);
        ability.increase(); // doesn't work because cap is 20
        assert_eq!(20, ability.get_score());
    }

    #[test]
    fn ability_scores_test() {
        let mut ability_scores = AbilityScores {
            strength: AbilityScore::new(15),
            dexterity: AbilityScore::new(12),
            constitution: AbilityScore::new(14),
            intelligence: AbilityScore::new(8),
            wisdom: AbilityScore::new(13),
            charisma: AbilityScore::new(10),
        };

        let str = ability_scores.get_score_mut(&Ability::STR);
        str.increase();
        let con = ability_scores.get_score_mut(&Ability::CON);
        con.increase();
        con.increase();
        let scores = ability_scores;
        assert_eq!(16, scores.get_score(&Ability::STR).get_score());
        assert_eq!(16, scores.get_score(&Ability::CON).get_score());
        assert_eq!(13, scores.get_score(&Ability::WIS).get_score());

        let other_scores = AbilityScores::new(16, 12, 16, 8, 13, 10);
        assert_eq!(scores, other_scores);
    }
}
