use std::fmt::{Display, Formatter};
use crate::ability_scores::Ability;
use crate::Character;

type CharacterDependant = Box<dyn Fn(&Character) -> i32>;

pub enum BonusType {
    Constant(i32),
    Modifier(Ability),
    Proficiency,
    Dependant(CharacterDependant),
}

pub struct BonusTerm {
    bonus: BonusType,
    name: Option<String>,
    attribution: Option<String>,
}

impl BonusTerm {
    pub fn new(value: BonusType) -> Self {
        BonusTerm {
            bonus: value,
            name: None,
            attribution: None,
        }
    }

    pub fn new_name(value: BonusType, name: String) -> Self {
        BonusTerm {
            bonus: value,
            name: Some(name),
            attribution: None,
        }
    }

    pub fn new_attr(value: BonusType, attr: String) -> Self {
        BonusTerm {
            bonus: value,
            name: None,
            attribution: Some(attr),
        }
    }

    pub fn new_name_attr(value: BonusType, name: String, attr: String) -> Self {
        BonusTerm {
            bonus: value,
            name: Some(name),
            attribution: Some(attr),
        }
    }

    pub fn get_name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    pub fn get_attr(&self) -> Option<&str> {
        self.attribution.as_deref()
    }

    pub fn get_bonus(&self) -> &BonusType {
        &self.bonus
    }
}

impl Display for BonusTerm {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.bonus {
            BonusType::Constant(c) => write!(f, "{}", c)?,
            BonusType::Modifier(a) => write!(f, "{}mod", a)?,
            BonusType::Proficiency => write!(f, "prof")?,
            BonusType::Dependant(_) => {
                if let Some(name) = &self.name {
                    write!(f, "{}", name)?;
                } else {
                    write!(f, "?")?;
                }
            }
        }
        if let Some(attr) = self.attribution.as_deref() {
            write!(f, " (From: {})", attr)?;
        }
        Ok(())
    }
}

pub struct AttributedBonus {
    name: String,
    terms: Vec<BonusTerm>
}

impl AttributedBonus {
    pub fn new(name: String) -> Self {
        AttributedBonus { name, terms: Vec::new() }
    }

    pub fn add_term(&mut self, term: BonusTerm) {
        self.terms.push(term);
    }

    pub fn get_value(&self, character: &Character) -> i32 {
        let mut bonus = 0;
        for term in self.terms.iter() {
            match term.get_bonus() {
                BonusType::Constant(c) => bonus += c,
                BonusType::Modifier(a) => bonus += character.get_ability_scores().get_score(a).get_mod() as i32,
                BonusType::Proficiency => bonus += character.get_prof_bonus() as i32,
                BonusType::Dependant(f) => bonus += f(character),
            };
        }
        bonus
    }
}

impl Display for AttributedBonus {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} = ", self.name)?;
        if self.terms.len() > 0 {
            write!(f, "{}", self.terms.get(0).unwrap())?;
        }
        let mut iter = self.terms.iter();
        iter.next();
        for term in iter {
            write!(f, " + {}", term)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::get_test_fighter;
    use super::*;

    #[test]
    fn attack_bonus_test() {
        let fighter = get_test_fighter();

        let mut to_hit = AttributedBonus::new(String::from("To Hit Bonus"));
        let str_mod: CharacterDependant = Box::new(|chr| chr.get_ability_scores().strength.get_mod() as i32);
        to_hit.add_term(BonusTerm::new_name(BonusType::Dependant(str_mod), String::from("str_mod")));
        let prof: CharacterDependant = Box::new(|chr| chr.get_prof_bonus() as i32);
        to_hit.add_term(BonusTerm::new_name(BonusType::Dependant(prof), String::from("prof")));
        assert_eq!(5, to_hit.get_value(&fighter));
        assert_eq!("To Hit Bonus = str_mod + prof", to_hit.to_string());

        let mut to_hit2 = AttributedBonus::new(String::from("To Hit Bonus"));
        to_hit2.add_term(BonusTerm::new(BonusType::Modifier(Ability::STR)));
        to_hit2.add_term(BonusTerm::new(BonusType::Proficiency));
        assert_eq!(5, to_hit2.get_value(&fighter));
        assert_eq!("To Hit Bonus = STRmod + prof", to_hit2.to_string());
    }

}
