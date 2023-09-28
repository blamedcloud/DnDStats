use std::rc::Rc;
use serde::{Deserialize, Serialize};
use combat_core::ability_scores::{Ability, AbilityScores};
use crate::{CBError, Character};
use crate::classes::{ChooseSubClass, ClassName, SubClass};
use crate::classes::fighter::ChampionFighter;
use crate::classes::ranger::HorizonWalkerRanger;
use crate::classes::rogue::{ArcaneTricksterRogue, ScoutRogue};
use crate::classes::wizard::ConjurationWizard;
use crate::equipment::{ACSource, Armor, ArmorName, Equipment, OffHand, Weapon, WeaponName};
use crate::feature::{AbilityScoreIncrease, ExtraAttack, Feature, SaveProficiencies};
use crate::feature::feats::{GreatWeaponMaster, PolearmMaster, Resilient, SharpShooter, ShieldMaster};
use crate::feature::fighting_style::{FightingStyle, FightingStyles};
use crate::spellcasting::cantrips::FireBoltCantrip;
use crate::spellcasting::fourth_lvl_spells::GreaterInvisibilitySpell;
use crate::spellcasting::third_lvl_spells::{FireBallSpell, HasteSpell};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CharacterDescription {
    name: String,
    base_ability_scores: [u8;6],
    equipment: EquipmentDescription,
    level_ups: Vec<LevelUp>,
}

impl CharacterDescription {
    pub fn new(name: String, scores: [u8;6], equipment: EquipmentDescription) -> Self {
        Self {
            name,
            base_ability_scores: scores,
            equipment,
            level_ups: Vec::new(),
        }
    }

    pub fn lvl_up(&mut self, lvl: LevelUp) {
        self.level_ups.push(lvl);
    }

    pub fn to_character(&self) -> Result<Character, CBError> {
        let mut character = Character::new(self.name.clone(), AbilityScores::from(self.base_ability_scores), self.equipment.to_equipment());
        for lvl_up in self.level_ups.iter() {
            let features = lvl_up.features.iter().map(|f_n| f_n.to_feature()).collect();
            character.level_up(lvl_up.class, features)?;
        }
        Ok(character)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EquipmentDescription {
    armor_name: ArmorName,
    armor_mb: Option<u8>,
    weapon_name: WeaponName,
    weapon_mb: Option<u8>,
    off_hand: OffHandDescription,
}

impl EquipmentDescription {
    pub fn new(an: ArmorName, wn: WeaponName, oh: OffHandDescription) -> Self {
        Self {
            armor_name: an,
            armor_mb: None,
            weapon_name: wn,
            weapon_mb: None,
            off_hand: oh,
        }
    }

    pub fn to_equipment(&self) -> Equipment {
        let mut armor = Armor::from(self.armor_name);
        if self.armor_mb.is_some() {
            armor.set_magic_bonus(self.armor_mb.unwrap());
        }
        let mut weapon = Weapon::from(self.weapon_name);
        if self.weapon_mb.is_some() {
            weapon.set_magic_bonus(self.weapon_mb.unwrap());
        }
        Equipment::new(armor, weapon, self.off_hand.to_offhand())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OffHandDescription {
    Weapon(WeaponName, Option<u8>),
    Shield(ACSource),
    Free,
}

impl OffHandDescription {
    pub fn to_offhand(&self) -> OffHand {
        match self {
            OffHandDescription::Weapon(wn, mb) => {
                if mb.is_some() {
                    let mut weapon = Weapon::from(*wn);
                    weapon.set_magic_bonus(mb.unwrap());
                    OffHand::Weapon(weapon)
                } else {
                    OffHand::Weapon(Weapon::from(*wn))
                }
            },
            OffHandDescription::Shield(acs) => OffHand::Shield(*acs),
            OffHandDescription::Free => OffHand::Free,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LevelUp {
    pub class: ClassName,
    pub features: Vec<FeatureName>,
}

impl LevelUp {
    pub fn feature(cn: ClassName, feat: FeatureName) -> Self {
        Self {
            class: cn,
            features: vec!(feat),
        }
    }

    pub fn new(cn: ClassName, fns: Vec<FeatureName>) -> Self {
        Self {
            class: cn,
            features: fns,
        }
    }
}

impl From<ClassName> for LevelUp {
    fn from(value: ClassName) -> Self {
        Self {
            class: value,
            features: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FeatureName {
    ASI(Ability, Ability),
    SaveProfs(Vec<Ability>),
    ExtraAttack(usize),
    FightingStyle(FightingStyles),
    GreatWeaponMaster,
    PolearmMaster,
    Resilient(Ability),
    SharpShooter,
    ShieldMaster,
    Subclass(SubClassName),
    FireBolt(Ability),
    Fireball(Ability),
    Haste,
    GreaterInvisibility,
}

impl FeatureName {
    pub fn to_feature(&self) -> Box<dyn Feature> {
        match self {
            FeatureName::ASI(ab1, ab2) => Box::new(AbilityScoreIncrease(*ab1, *ab2)),
            FeatureName::SaveProfs(abs) => Box::new(SaveProficiencies::from(abs.clone())),
            FeatureName::ExtraAttack(aa) => Box::new(ExtraAttack(*aa)),
            FeatureName::FightingStyle(fs) => Box::new(FightingStyle(*fs)),
            FeatureName::GreatWeaponMaster => Box::new(GreatWeaponMaster),
            FeatureName::PolearmMaster => Box::new(PolearmMaster),
            FeatureName::Resilient(ab) => Box::new(Resilient(*ab)),
            FeatureName::SharpShooter => Box::new(SharpShooter),
            FeatureName::ShieldMaster => Box::new(ShieldMaster),
            FeatureName::Subclass(scn) => Box::new(ChooseSubClass(scn.to_subclass())),
            FeatureName::FireBolt(ab) => Box::new(FireBoltCantrip(*ab)),
            FeatureName::Fireball(ab) => Box::new(FireBallSpell(*ab)),
            FeatureName::Haste => Box::new(HasteSpell),
            FeatureName::GreaterInvisibility => Box::new(GreaterInvisibilitySpell),
        }
    }
}

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone, Serialize, Deserialize)]
pub enum SubClassName {
    ChampionFighter,
    HorizonWalkerRanger,
    ScoutRogue,
    ArcaneTricksterRogue,
    ConjurationWizard,
}

impl SubClassName {
    pub fn to_subclass(&self) -> Rc<dyn SubClass> {
        match self {
            SubClassName::ChampionFighter => Rc::new(ChampionFighter),
            SubClassName::HorizonWalkerRanger => Rc::new(HorizonWalkerRanger),
            SubClassName::ScoutRogue => Rc::new(ScoutRogue),
            SubClassName::ArcaneTricksterRogue => Rc::new(ArcaneTricksterRogue),
            SubClassName::ConjurationWizard => Rc::new(ConjurationWizard),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use combat_core::ability_scores::Ability;
    use crate::classes::ClassName;
    use crate::equipment::{ArmorName, WeaponName};
    use crate::serialization::{CharacterDescription, EquipmentDescription, FeatureName, LevelUp, OffHandDescription, SubClassName};

    macro_rules! test_case {($fname:expr) => (
        concat!(env!("CARGO_MANIFEST_DIR"), "/resources/test/", $fname)
    )}

    #[test]
    fn json_test() {
        let scores = [10,10,10,10,10,10];
        let equipment = EquipmentDescription::new(ArmorName::Leather, WeaponName::Dagger, OffHandDescription::Weapon(WeaponName::Dagger, None));
        let mut basic_char = CharacterDescription::new(String::from("basic"), scores, equipment);
        basic_char.lvl_up(LevelUp::from(ClassName::Rogue));
        basic_char.lvl_up(LevelUp::from(ClassName::Rogue));
        basic_char.lvl_up(LevelUp::feature(ClassName::Rogue, FeatureName::Subclass(SubClassName::ScoutRogue)));

        let char_json = serde_json::to_string(&basic_char).unwrap();
        assert_eq!(char_json, "{\"name\":\"basic\",\"base_ability_scores\":[10,10,10,10,10,10],\"equipment\":{\"armor_name\":\"Leather\",\"armor_mb\":null,\"weapon_name\":\"Dagger\",\"weapon_mb\":null,\"off_hand\":{\"Weapon\":[\"Dagger\",null]}},\"level_ups\":[{\"class\":\"Rogue\",\"features\":[]},{\"class\":\"Rogue\",\"features\":[]},{\"class\":\"Rogue\",\"features\":[{\"Subclass\":\"ScoutRogue\"}]}]}");

        let char_copy: CharacterDescription = serde_json::from_str(&char_json).unwrap();
        assert_eq!(basic_char, char_copy);
    }

    #[test]
    fn file_read_test() {
        let scores = [8,16,14,16,12,10];
        let equipment = EquipmentDescription::new(
            ArmorName::MageArmor,
            WeaponName::Rapier,
            OffHandDescription::Free
        );
        let mut character = CharacterDescription::new(String::from("harry"), scores, equipment);
        character.lvl_up(LevelUp::from(ClassName::Wizard));
        character.lvl_up(LevelUp::feature(ClassName::Wizard, FeatureName::Subclass(SubClassName::ConjurationWizard)));
        character.lvl_up(LevelUp::from(ClassName::Wizard));
        character.lvl_up(LevelUp::feature(ClassName::Wizard, FeatureName::ASI(Ability::INT, Ability::INT)));
        character.lvl_up(LevelUp::feature(ClassName::Wizard, FeatureName::Fireball(Ability::INT)));

        let file = fs::File::open(test_case!("wizard_harry.json")).unwrap();
        let file_char: CharacterDescription = serde_json::from_reader(file).unwrap();
        assert_eq!(character, file_char);
    }
}
