use std::fmt::{Display, Formatter};

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
        }
        write!(f, "{}", s)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct AbilityScore {
    score: u8,
    prof_save: bool,
    save_bonus: i8,
}

impl AbilityScore {
    pub fn new(score: u8) -> Self {
        AbilityScore { score, prof_save: false, save_bonus: 0 }
    }

    pub fn get_score(&self) -> u8 {
        self.score
    }

    pub fn get_mod(&self) -> i8 {
        ((self.score/2) as i8) - 5
    }

    pub fn increase(&mut self) {
        self.score += 1;
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
}

#[derive(Debug, PartialEq, Eq)]
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
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_ability_test() {
        let ability = AbilityScore { score: 1, prof_save: false, save_bonus: 0 };
        assert_eq!(ability.get_mod(), -5);
        let ability = AbilityScore { score: 10, prof_save: false, save_bonus: 0 };
        assert_eq!(ability.get_mod(), 0);
        let ability = AbilityScore { score: 13, prof_save: false, save_bonus: 0 };
        assert_eq!(ability.get_mod(), 1);
        let mut ability = AbilityScore { score: 16, prof_save: false, save_bonus: 0 };
        assert_eq!(ability.get_mod(), 3);
        ability.increase(); // 17
        assert_eq!(ability.get_mod(), 3);
        ability.increase(); // 18
        assert_eq!(ability.get_mod(), 4);
        let ability = AbilityScore { score: 20, prof_save: false, save_bonus: 0 };
        assert_eq!(ability.get_mod(), 5);
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