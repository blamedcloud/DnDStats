use std::fmt::{Debug, Display, Formatter};
use std::rc::Rc;
use crate::ability_scores::Ability;
use crate::Character;
use crate::classes::ClassName;

pub type CharacterDependant = Rc<dyn Fn(&Character) -> i32>;

#[derive(Clone)]
pub enum BonusType {
    Constant(i32),
    Modifier(Ability),
    Proficiency,
    ClassLevel(ClassName),
    Dependant(CharacterDependant),
}

#[derive(Clone)]
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

    pub fn get_value(&self, character: &Character) -> i32 {
        let value = match &self.bonus {
            BonusType::Constant(c) => *c,
            BonusType::Modifier(a) => character.get_ability_scores().get_score(a).get_mod() as i32,
            BonusType::Proficiency => character.get_prof_bonus() as i32,
            BonusType::ClassLevel(cn) => character.get_class_level(*cn) as i32,
            BonusType::Dependant(f) => f(character),
        };
        value
    }
}

impl Display for BonusTerm {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.bonus {
            BonusType::Constant(c) => write!(f, "{}", c)?,
            BonusType::Modifier(a) => write!(f, "{}mod", a)?,
            BonusType::Proficiency => write!(f, "prof")?,
            BonusType::ClassLevel(cn) => write!(f, "{} level", *cn)?,
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

impl Debug for BonusTerm {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

#[derive(Debug, Clone)]
pub struct AttributedBonus {
    name: String, // TODO: rather than use a string, this might be better as an enum eventually?
    terms: Vec<BonusTerm>,
    temp_terms: Vec<BonusTerm>,
    cached_value: Option<i32>,
}

impl AttributedBonus {
    pub fn new(name: String) -> Self {
        AttributedBonus { name, terms: Vec::new(), temp_terms: Vec::new(), cached_value: None }
    }

    pub fn reset(&mut self) {
        self.terms.clear();
        self.temp_terms.clear();
    }

    pub fn add_term(&mut self, term: BonusTerm) {
        self.terms.push(term);
    }

    pub fn add_temp_term(&mut self, term: BonusTerm) {
        self.temp_terms.push(term);
    }

    pub fn clear_temp_terms(&mut self) {
        self.temp_terms.clear();
    }

    pub fn get_value(&self, character: &Character) -> i32 {
        let mut bonus = 0;
        for term in self.terms.iter() {
            bonus += term.get_value(character);
        }
        for term in self.temp_terms.iter() {
            bonus += term.get_value(character);
        }
        bonus
    }

    pub fn save_value(&mut self, character: &Character) {
        self.cached_value = Some(self.get_value(character));
    }

    pub fn clear_saved_value(&mut self) {
        self.cached_value = None;
    }

    pub fn get_saved_value(&self) -> Option<i32> {
        self.cached_value
    }
}

impl Display for AttributedBonus {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} = ", self.name)?;

        // write each term, careful about the first one (no leading '+')
        if self.terms.len() > 0 {
            write!(f, "{}", self.terms.get(0).unwrap())?;
        }
        let mut iter = self.terms.iter();
        iter.next();
        for term in iter {
            write!(f, " + {}", term)?;
        }

        // write any temporary terms
        for term in self.temp_terms.iter() {
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
        let str_mod: CharacterDependant = Rc::new(|chr| chr.get_ability_scores().strength.get_mod() as i32);
        to_hit.add_term(BonusTerm::new_name(BonusType::Dependant(str_mod), String::from("str_mod")));
        let prof: CharacterDependant = Rc::new(|chr| chr.get_prof_bonus() as i32);
        to_hit.add_term(BonusTerm::new_name(BonusType::Dependant(prof), String::from("prof")));
        assert_eq!(5, to_hit.get_value(&fighter));
        assert_eq!("To Hit Bonus = str_mod + prof", to_hit.to_string());

        let mut to_hit2 = AttributedBonus::new(String::from("To Hit Bonus"));
        to_hit2.add_term(BonusTerm::new(BonusType::Modifier(Ability::STR)));
        to_hit2.add_term(BonusTerm::new(BonusType::Proficiency));
        assert_eq!(5, to_hit2.get_value(&fighter));
        assert_eq!("To Hit Bonus = STRmod + prof", to_hit2.to_string());
    }

    #[test]
    fn attributed_bonus_clone_test() {
        let fighter = get_test_fighter();
        let ac = &fighter.armor_class;
        let ac_clone = ac.clone();
        assert_eq!(ac.get_value(&fighter), ac_clone.get_value(&fighter));
        assert_eq!(ac.to_string(), ac_clone.to_string())
    }

}
