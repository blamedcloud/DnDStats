use crate::Character;
use crate::classes::SpellCasterType;

pub fn character_spell_slots(character: &Character) -> [usize;10] {
    let caster_level = character_caster_level(character);
    spell_slots_by_caster_level(caster_level)
}

pub fn character_caster_level(character: &Character) -> u8 {
    let mut eff_caster_level = 0;
    for class in character.get_class_levels() {
        let mut caster_type = class.get_default_spellcasting();
        if let Some(sc) = character.get_sub_class(*class) {
            sc.get_spellcasting_override().map(|sct| caster_type = sct);
        }
        match caster_type {
            SpellCasterType::Martial => {}
            SpellCasterType::ThirdCaster => eff_caster_level += 2,
            SpellCasterType::HalfCaster => eff_caster_level += 3,
            SpellCasterType::FullCaster => eff_caster_level += 6,
        }
    }
    let remainder = eff_caster_level % 6;
    eff_caster_level /= 6;
    // half and third casters don't get spell-casting
    // until level 2 or 3 (eff_caster_level > 0)
    // but after that point, it effectively rounds up
    if remainder != 0 && eff_caster_level > 0 {
        eff_caster_level += 1;
    }
    eff_caster_level
}

pub fn spell_slots_by_caster_level(caster_level: u8) -> [usize;10] {
    // index 0 is technically for cantrips. This allows
    // each slot level to be its own index.
    let mut slots = [0; 10];
    slots[1] = first_level_slots(caster_level);
    slots[2] = second_level_slots(caster_level);
    slots[3] = third_level_slots(caster_level);
    slots[4] = fourth_level_slots(caster_level);
    slots[5] = fifth_level_slots(caster_level);
    slots[6] = sixth_level_slots(caster_level);
    slots[7] = seventh_level_slots(caster_level);
    slots[8] = eighth_level_slots(caster_level);
    slots[9] = ninth_level_slots(caster_level);
    slots
}

pub fn first_level_slots(caster_level: u8) -> usize {
    let mut slots = 0;
    if caster_level == 1 {
        slots = 2;
    } else if caster_level == 2 {
        slots = 3;
    } else if caster_level >= 3 {
        slots = 4;
    }
    slots
}

pub fn second_level_slots(caster_level: u8) -> usize {
    let mut slots = 0;
    if caster_level == 3 {
        slots = 2;
    } else if caster_level >= 4 {
        slots = 3;
    }
    slots
}

pub fn third_level_slots(caster_level: u8) -> usize {
    let mut slots = 0;
    if caster_level == 5 {
        slots = 2;
    } else if caster_level >= 6 {
        slots = 3;
    }
    slots
}

pub fn fourth_level_slots(caster_level: u8) -> usize {
    let mut slots = 0;
    if caster_level == 7 {
        slots = 1;
    } else if caster_level == 8 {
        slots = 2;
    } else if caster_level >= 9 {
        slots = 3;
    }
    slots
}

pub fn fifth_level_slots(caster_level: u8) -> usize {
    let mut slots = 0;
    if caster_level == 9 {
        slots = 1;
    } else if caster_level > 9 && caster_level < 18 {
        slots = 2;
    } else if caster_level >= 18 {
        slots = 3;
    }
    slots
}

pub fn sixth_level_slots(caster_level: u8) -> usize {
    let mut slots = 0;
    if caster_level >= 11 && caster_level < 19 {
        slots = 1;
    } else if caster_level >= 19 {
        slots = 2;
    }
    slots
}

pub fn seventh_level_slots(caster_level: u8) -> usize {
    let mut slots = 0;
    if caster_level >= 13 && caster_level < 20 {
        slots = 1;
    } else if caster_level >= 20 {
        slots = 2;
    }
    slots
}

pub fn eighth_level_slots(caster_level: u8) -> usize {
    let mut slots = 0;
    if caster_level >= 15 {
        slots = 1;
    }
    slots
}

pub fn ninth_level_slots(caster_level: u8) -> usize {
    let mut slots = 0;
    if caster_level >= 17 {
        slots = 1;
    }
    slots
}

#[cfg(test)]
mod tests {
    use combat_core::ability_scores::AbilityScores;
    use crate::Character;
    use crate::classes::{ChooseSubClass, ClassName};
    use crate::classes::ranger::HorizonWalkerRanger;
    use crate::classes::rogue::ArcaneTricksterRogue;
    use crate::classes::wizard::ConjurationWizard;
    use crate::equipment::{Armor, Equipment, OffHand, Weapon};
    use crate::spellcasting::{character_caster_level, spell_slots_by_caster_level};

    fn level0_character() -> Character {
        let name = String::from("caster");
        let ability_scores = AbilityScores::new(12, 12, 12, 12, 12, 12);
        let equipment = Equipment::new(
            Armor::no_armor(),
            Weapon::quarterstaff(),
            OffHand::Free,
        );
        Character::new(name, ability_scores, equipment)
    }

    #[test]
    fn spell_slots() {
        assert_eq!(spell_slots_by_caster_level(1), [0,2,0,0,0,0,0,0,0,0]);
        assert_eq!(spell_slots_by_caster_level(2), [0,3,0,0,0,0,0,0,0,0]);
        assert_eq!(spell_slots_by_caster_level(3), [0,4,2,0,0,0,0,0,0,0]);
        assert_eq!(spell_slots_by_caster_level(4), [0,4,3,0,0,0,0,0,0,0]);
        assert_eq!(spell_slots_by_caster_level(5), [0,4,3,2,0,0,0,0,0,0]);
        assert_eq!(spell_slots_by_caster_level(6), [0,4,3,3,0,0,0,0,0,0]);
        assert_eq!(spell_slots_by_caster_level(7), [0,4,3,3,1,0,0,0,0,0]);
        assert_eq!(spell_slots_by_caster_level(8), [0,4,3,3,2,0,0,0,0,0]);
        assert_eq!(spell_slots_by_caster_level(9), [0,4,3,3,3,1,0,0,0,0]);
        assert_eq!(spell_slots_by_caster_level(10), [0,4,3,3,3,2,0,0,0,0]);
        assert_eq!(spell_slots_by_caster_level(11), [0,4,3,3,3,2,1,0,0,0]);
        assert_eq!(spell_slots_by_caster_level(12), [0,4,3,3,3,2,1,0,0,0]);
        assert_eq!(spell_slots_by_caster_level(13), [0,4,3,3,3,2,1,1,0,0]);
        assert_eq!(spell_slots_by_caster_level(14), [0,4,3,3,3,2,1,1,0,0]);
        assert_eq!(spell_slots_by_caster_level(15), [0,4,3,3,3,2,1,1,1,0]);
        assert_eq!(spell_slots_by_caster_level(16), [0,4,3,3,3,2,1,1,1,0]);
        assert_eq!(spell_slots_by_caster_level(17), [0,4,3,3,3,2,1,1,1,1]);
        assert_eq!(spell_slots_by_caster_level(18), [0,4,3,3,3,3,1,1,1,1]);
        assert_eq!(spell_slots_by_caster_level(19), [0,4,3,3,3,3,2,1,1,1]);
        assert_eq!(spell_slots_by_caster_level(20), [0,4,3,3,3,3,2,2,1,1]);
    }

    #[test]
    fn pure_wizard() {
        let mut wizard = level0_character();
        wizard.level_up(ClassName::Wizard, vec!()).unwrap();
        assert_eq!(1, character_caster_level(&wizard));
        wizard.level_up(ClassName::Wizard, vec!(Box::new(ChooseSubClass(ConjurationWizard)))).unwrap();
        assert_eq!(2, character_caster_level(&wizard));
        wizard.level_up_basic().unwrap();
        assert_eq!(3, character_caster_level(&wizard));
        wizard.level_up_basic().unwrap();
        assert_eq!(4, character_caster_level(&wizard));
        wizard.level_up_basic().unwrap();
        assert_eq!(5, character_caster_level(&wizard));
        wizard.level_up_basic().unwrap();
        assert_eq!(6, character_caster_level(&wizard));
    }

    #[test]
    fn pure_ranger() {
        let mut ranger = level0_character();
        ranger.level_up(ClassName::Ranger, vec!()).unwrap();
        assert_eq!(0, character_caster_level(&ranger));
        ranger.level_up_basic().unwrap();
        assert_eq!(1, character_caster_level(&ranger));
        ranger.level_up(ClassName::Ranger, vec!(Box::new(ChooseSubClass(HorizonWalkerRanger)))).unwrap();
        assert_eq!(2, character_caster_level(&ranger));
        ranger.level_up_basic().unwrap();
        assert_eq!(2, character_caster_level(&ranger));
        ranger.level_up_basic().unwrap();
        assert_eq!(3, character_caster_level(&ranger));
        ranger.level_up_basic().unwrap();
        assert_eq!(3, character_caster_level(&ranger));
        ranger.level_up_basic().unwrap();
        assert_eq!(4, character_caster_level(&ranger));
        ranger.level_up_basic().unwrap();
        assert_eq!(4, character_caster_level(&ranger));
        ranger.level_up_basic().unwrap();
        assert_eq!(5, character_caster_level(&ranger));
    }

    #[test]
    fn pure_arcane_trickster() {
        let mut rogue = level0_character();
        rogue.level_up(ClassName::Rogue, vec!()).unwrap();
        assert_eq!(0, character_caster_level(&rogue));
        rogue.level_up_basic().unwrap();
        assert_eq!(0, character_caster_level(&rogue));
        rogue.level_up(ClassName::Rogue, vec!(Box::new(ChooseSubClass(ArcaneTricksterRogue)))).unwrap();
        assert_eq!(1, character_caster_level(&rogue));
        rogue.level_up_basic().unwrap();
        assert_eq!(2, character_caster_level(&rogue));
        rogue.level_up_basic().unwrap();
        assert_eq!(2, character_caster_level(&rogue));
        rogue.level_up_basic().unwrap();
        assert_eq!(2, character_caster_level(&rogue));
        rogue.level_up_basic().unwrap();
        assert_eq!(3, character_caster_level(&rogue));
        rogue.level_up_basic().unwrap();
        assert_eq!(3, character_caster_level(&rogue));
        rogue.level_up_basic().unwrap();
        assert_eq!(3, character_caster_level(&rogue));
        rogue.level_up_basic().unwrap();
        assert_eq!(4, character_caster_level(&rogue));
    }
}
