use combat_core::damage::{DamageDice, DamageType};
use combat_core::movement::Feet;

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum ArmorType {
    NoArmor,
    LightArmor,
    MediumArmor,
    HeavyArmor,
}

#[derive(Debug, Clone)]
pub struct Armor {
    armor_type: ArmorType,
    name: String,
    ac: ACSource,
}

impl Armor {
    pub fn no_armor() -> Self {
        Armor {
            armor_type: ArmorType::NoArmor,
            name: String::from("No Armor"),
            ac: ACSource { base_ac: 10, magic_bonus_ac: None }
        }
    }

    pub fn mage_armor() -> Self {
        Armor {
            armor_type: ArmorType::NoArmor,
            name: String::from("Mage Armor"),
            ac: ACSource { base_ac: 13, magic_bonus_ac: None }
        }
    }

    pub fn padded() -> Self {
        Armor {
            armor_type: ArmorType::LightArmor,
            name: String::from("Padded"),
            ac: ACSource { base_ac: 11, magic_bonus_ac: None }
        }
    }

    pub fn leather() -> Self {
        Armor {
            armor_type: ArmorType::LightArmor,
            name: String::from("Leather"),
            ac: ACSource { base_ac: 11, magic_bonus_ac: None }
        }
    }

    pub fn studded_leather() -> Self {
        Armor {
            armor_type: ArmorType::LightArmor,
            name: String::from("Studded Leather"),
            ac: ACSource { base_ac: 12, magic_bonus_ac: None }
        }
    }

    pub fn hide() -> Self {
        Armor {
            armor_type: ArmorType::MediumArmor,
            name: String::from("Hide"),
            ac: ACSource { base_ac: 12, magic_bonus_ac: None }
        }
    }

    pub fn chain_shirt() -> Self {
        Armor {
            armor_type: ArmorType::MediumArmor,
            name: String::from("Chain Shirt"),
            ac: ACSource { base_ac: 13, magic_bonus_ac: None }
        }
    }

    pub fn scale_mail() -> Self {
        Armor {
            armor_type: ArmorType::MediumArmor,
            name: String::from("Scale Mail"),
            ac: ACSource { base_ac: 14, magic_bonus_ac: None }
        }
    }

    pub fn breastplate() -> Self {
        Armor {
            armor_type: ArmorType::MediumArmor,
            name: String::from("Breastplate"),
            ac: ACSource { base_ac: 14, magic_bonus_ac: None }
        }
    }

    pub fn half_plate() -> Self {
        Armor {
            armor_type: ArmorType::MediumArmor,
            name: String::from("Half Plate"),
            ac: ACSource { base_ac: 15, magic_bonus_ac: None }
        }
    }

    pub fn ring_mail() -> Self {
        Armor {
            armor_type: ArmorType::HeavyArmor,
            name: String::from("Ring Mail"),
            ac: ACSource { base_ac: 14, magic_bonus_ac: None }
        }
    }

    pub fn chain_mail() -> Self {
        Armor {
            armor_type: ArmorType::HeavyArmor,
            name: String::from("Chain Mail"),
            ac: ACSource { base_ac: 16, magic_bonus_ac: None }
        }
    }

    pub fn splint() -> Self {
        Armor {
            armor_type: ArmorType::HeavyArmor,
            name: String::from("Splint"),
            ac: ACSource { base_ac: 17, magic_bonus_ac: None }
        }
    }

    pub fn plate() -> Self {
        Armor {
            armor_type: ArmorType::HeavyArmor,
            name: String::from("Plate"),
            ac: ACSource { base_ac: 18, magic_bonus_ac: None }
        }
    }

    pub fn get_armor_type(&self) -> &ArmorType {
        &self.armor_type
    }

    pub fn get_armor_name(&self) -> &str {
        self.name.as_str()
    }

    pub fn get_ac_source(&self) -> &ACSource {
        &self.ac
    }

    pub fn set_magic_bonus(&mut self, value: u8) {
        self.ac.set_magic_bonus(value);
    }

    pub fn unset_magic_bonus(&mut self) {
        self.ac.unset_magic_bonus();
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct ACSource {
    base_ac: u8,
    magic_bonus_ac: Option<u8>,
}

impl ACSource {
    pub fn shield() -> Self {
        ACSource { base_ac: 2, magic_bonus_ac: None }
    }

    pub fn get_base_ac(&self) -> u8 {
        self.base_ac
    }

    pub fn get_magic_bonus(&self) -> Option<u8> {
        self.magic_bonus_ac
    }

    pub fn set_magic_bonus(&mut self, value: u8) {
        self.magic_bonus_ac = Some(value);
    }

    pub fn unset_magic_bonus(&mut self) {
        self.magic_bonus_ac = None;
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum WeaponType {
    SimpleMelee,
    SimpleRanged,
    MartialMelee,
    MartialRanged,
}

impl WeaponType {
    pub fn is_ranged(&self) -> bool {
        match self {
            WeaponType::SimpleMelee => false,
            WeaponType::SimpleRanged => true,
            WeaponType::MartialMelee => false,
            WeaponType::MartialRanged => true,
        }
    }
    pub fn is_melee(&self) -> bool {
        !self.is_ranged()
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum WeaponProperty {
    Ammunition,
    Finesse,
    Heavy,
    Light,
    Loading,
    Range(Feet,Feet),
    Reach,
    Special,
    Thrown,
    TwoHanded,
    Versatile(DamageDice),
}

#[derive(Debug, PartialEq)]
pub enum WeaponRange {
    Melee(Feet),
    Ranged(Feet,Feet),
    Thrown(Feet, Feet, Feet),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Weapon {
    name: String,
    weapon_type: WeaponType,
    dice: DamageDice,
    dmg_type: DamageType,
    properties: Vec<WeaponProperty>,
    magic_bonus: Option<u8>,
}

impl Weapon { // TODO: implement the rest of the weapon constructors
    pub fn dagger() -> Self {
        Weapon {
            name: String::from("Dagger"),
            weapon_type: WeaponType::SimpleMelee,
            dice: DamageDice::D4,
            dmg_type: DamageType::Piercing,
            properties: vec!(WeaponProperty::Finesse, WeaponProperty::Light, WeaponProperty::Thrown, WeaponProperty::Range(Feet(20),Feet(60))),
            magic_bonus: None,
        }
    }

    pub const QUARTERSTAFF: &'static str = "Quarterstaff";
    pub fn quarterstaff() -> Self {
        Weapon {
            name: String::from(Weapon::QUARTERSTAFF),
            weapon_type: WeaponType::SimpleMelee,
            dice: DamageDice::D6,
            dmg_type: DamageType::Bludgeoning,
            properties: vec!(WeaponProperty::Versatile(DamageDice::D8)),
            magic_bonus: None,
        }
    }

    pub fn shortbow() -> Self {
        Weapon {
            name: String::from("Shortbow"),
            weapon_type: WeaponType::SimpleRanged,
            dice: DamageDice::D6,
            dmg_type: DamageType::Piercing,
            properties: vec!(WeaponProperty::Ammunition, WeaponProperty::Range(Feet(80),Feet(320)), WeaponProperty::TwoHanded),
            magic_bonus: None,
        }
    }

    pub const GLAIVE: &'static str = "Glaive";
    pub fn glaive() -> Self {
        Weapon {
            name: String::from(Weapon::GLAIVE),
            weapon_type: WeaponType::MartialMelee,
            dice: DamageDice::D10,
            dmg_type: DamageType::Slashing,
            properties: vec!(WeaponProperty::Heavy, WeaponProperty::Reach, WeaponProperty::TwoHanded),
            magic_bonus: None,
        }
    }

    pub fn greatsword() -> Self {
        Weapon {
            name: String::from("Greatsword"),
            weapon_type: WeaponType::MartialMelee,
            dice: DamageDice::TwoD6,
            dmg_type: DamageType::Slashing,
            properties: vec!(WeaponProperty::Heavy, WeaponProperty::TwoHanded),
            magic_bonus: None,
        }
    }

    pub const HALBERD: &'static str = "Halberd";
    pub fn halberd() -> Self {
        Weapon {
            name: String::from(Weapon::HALBERD),
            weapon_type: WeaponType::MartialMelee,
            dice: DamageDice::D10,
            dmg_type: DamageType::Slashing,
            properties: vec!(WeaponProperty::Heavy, WeaponProperty::Reach, WeaponProperty::TwoHanded),
            magic_bonus: None,
        }
    }

    pub fn longsword() -> Self {
        Weapon {
            name: String::from("Longsword"),
            weapon_type: WeaponType::MartialMelee,
            dice: DamageDice::D8,
            dmg_type: DamageType::Slashing,
            properties: vec!(WeaponProperty::Versatile(DamageDice::D10)),
            magic_bonus: None,
        }
    }

    pub fn rapier() -> Self {
        Weapon {
            name: String::from("Rapier"),
            weapon_type: WeaponType::MartialMelee,
            dice: DamageDice::D8,
            dmg_type: DamageType::Piercing,
            properties: vec!(WeaponProperty::Finesse),
            magic_bonus: None,
        }
    }

    pub fn shortsword() -> Self {
        Weapon {
            name: String::from("Shortsword"),
            weapon_type: WeaponType::MartialMelee,
            dice: DamageDice::D6,
            dmg_type: DamageType::Piercing,
            properties: vec!(WeaponProperty::Finesse, WeaponProperty::Light),
            magic_bonus: None,
        }
    }

    pub fn longbow() -> Self {
        Weapon {
            name: String::from("Longbow"),
            weapon_type: WeaponType::MartialRanged,
            dice: DamageDice::D8,
            dmg_type: DamageType::Piercing,
            properties: vec!(WeaponProperty::Ammunition, WeaponProperty::Heavy, WeaponProperty::TwoHanded, WeaponProperty::Range(Feet(150), Feet(600))),
            magic_bonus: None,
        }
    }

    pub fn as_pam(&self) -> Self {
        let mut pam = self.clone();
        pam.name.push_str(" PAM");
        pam.dice = DamageDice::D4;
        pam.dmg_type = DamageType::Bludgeoning;
        pam
    }

    pub fn get_name(&self) -> &str {
        self.name.as_str()
    }

    pub fn get_type(&self) -> &WeaponType {
        &self.weapon_type
    }

    pub fn get_dice(&self) -> &DamageDice {
        &self.dice
    }

    pub fn get_dmg_type(&self) -> &DamageType {
        &self.dmg_type
    }

    pub fn get_properties(&self) -> &Vec<WeaponProperty> {
        &self.properties
    }

    pub fn has_property(&self, prop: WeaponProperty) -> bool {
        self.properties.contains(&prop)
    }

    pub fn get_magic_bonus(&self) -> Option<u8> {
        self.magic_bonus
    }

    pub fn set_magic_bonus(&mut self, value: u8) {
        self.magic_bonus = Some(value);
    }

    pub fn unset_magic_bonus(&mut self) {
        self.magic_bonus = None;
    }

    pub fn is_versatile(&self) -> Option<&DamageDice> {
        for prop in self.properties.iter() {
            if let WeaponProperty::Versatile(dice) = prop {
                return Some(dice);
            }
        }
        None
    }

    pub fn get_range(&self) -> WeaponRange {
        let mut melee_range = Feet(5);
        if self.properties.contains(&WeaponProperty::Reach) {
            melee_range = Feet(10);
        }
        for prop in self.properties.iter() {
            if let WeaponProperty::Range(short, long) = prop {
                return if self.properties.contains(&WeaponProperty::Thrown) {
                    WeaponRange::Thrown(melee_range, *short, *long)
                } else {
                    WeaponRange::Ranged(*short, *long)
                }
            }
        }
        WeaponRange::Melee(melee_range)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum OffHand {
    Weapon(Weapon),
    Shield(ACSource),
    Free,
}

#[derive(Debug, Clone)]
pub struct Equipment {
    armor: Armor,
    main_hand: Weapon,
    off_hand: OffHand,
}

impl Equipment {
    pub fn new(armor: Armor, main: Weapon, off: OffHand) -> Self {
        Equipment {
            armor,
            main_hand: main,
            off_hand: off,
        }
    }

    pub fn get_armor(&self) -> &Armor {
        &self.armor
    }
    pub fn get_armor_mut(&mut self) -> &mut Armor {
        &mut self.armor
    }
    pub fn set_armor(&mut self, new_armor: Armor) {
        self.armor = new_armor;
    }

    pub fn get_primary_weapon(&self) -> &Weapon {
        &self.main_hand
    }
    pub fn get_primary_weapon_mut(&mut self) -> &mut Weapon {
        &mut self.main_hand
    }
    pub fn set_primary_weapon(&mut self, weapon: Weapon) {
        self.main_hand = weapon;
    }

    pub fn get_primary_dmg_dice(&self) -> &DamageDice {
        let versatile = self.get_primary_weapon().is_versatile();
        if let Some(dice) = versatile {
            if OffHand::Free == self.off_hand {
                return dice;
            }
        }
        self.get_primary_weapon().get_dice()
    }

    pub fn get_off_hand(&self) -> &OffHand {
        &self.off_hand
    }
    pub fn get_off_hand_mut(&mut self) -> &mut OffHand {
        &mut self.off_hand
    }
    pub fn set_off_hand(&mut self, off: OffHand) {
        self.off_hand = off;
    }

    pub fn get_secondary_weapon(&self) -> Option<&Weapon> {
        if let OffHand::Weapon(w) = &self.off_hand {
            Some(w)
        } else {
            None
        }
    }
    pub fn get_shield(&self) -> Option<&ACSource> {
        if let OffHand::Shield(s) = &self.off_hand {
            Some(s)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_equip_test() {
        let equip = Equipment::new(
            Armor::studded_leather(),
            Weapon::longbow(),
            OffHand::Free
        );
        assert_eq!(12, equip.get_armor().get_ac_source().get_base_ac());
        assert_eq!(None, equip.get_armor().get_ac_source().get_magic_bonus());
        assert_eq!(&ArmorType::LightArmor, equip.get_armor().get_armor_type());

        assert_eq!(&WeaponType::MartialRanged, equip.get_primary_weapon().get_type());
        assert_eq!(&DamageDice::D8, equip.get_primary_weapon().get_dice());
        assert_eq!(&DamageType::Piercing, equip.get_primary_weapon().get_dmg_type());
        assert_eq!(None, equip.get_primary_weapon().get_magic_bonus());
        let properties = equip.get_primary_weapon().get_properties();
        assert_eq!(4, properties.len());
        assert!(properties.contains(&WeaponProperty::Ammunition));
        assert!(properties.contains(&WeaponProperty::Heavy));
        assert!(properties.contains(&WeaponProperty::TwoHanded));
        assert!(properties.contains(&WeaponProperty::Range(Feet(150), Feet(600))));
        assert_eq!(WeaponRange::Ranged(Feet(150), Feet(600)), equip.get_primary_weapon().get_range());

        assert_eq!(&OffHand::Free, equip.get_off_hand());
        assert_eq!(None, equip.get_secondary_weapon());
    }

    #[test]
    fn magic_upgrade_test() {
        let mut equip = Equipment::new(
            Armor::plate(),
            Weapon::longsword(),
            OffHand::Shield(ACSource::shield())
        );
        assert_eq!(None, equip.get_armor().get_ac_source().get_magic_bonus());
        equip.get_armor_mut().set_magic_bonus(3);
        assert_eq!(3, equip.get_armor().get_ac_source().get_magic_bonus().unwrap());
        assert_eq!(None, equip.get_primary_weapon().get_magic_bonus());
        equip.get_primary_weapon_mut().set_magic_bonus(1);
        assert_eq!(1, equip.get_primary_weapon().get_magic_bonus().unwrap());
        assert_eq!(None, equip.get_secondary_weapon());
        if let OffHand::Shield(shield) = equip.get_off_hand_mut() {
            assert_eq!(None, shield.get_magic_bonus());
            shield.set_magic_bonus(2);
            assert_eq!(2, shield.get_magic_bonus().unwrap());
        }
        assert_eq!(2, equip.get_shield().unwrap().get_magic_bonus().unwrap());
    }

    #[test]
    fn range_test() {
        let equip1 = Equipment::new(
            Armor::leather(),
            Weapon::shortsword(),
            OffHand::Weapon(Weapon::dagger()),
        );
        assert_eq!(WeaponRange::Melee(Feet(5)), equip1.get_primary_weapon().get_range());
        assert_eq!(WeaponRange::Thrown(Feet(5), Feet(20), Feet(60)), equip1.get_secondary_weapon().unwrap().get_range());

        let equip2 = Equipment::new(
            Armor::chain_mail(),
            Weapon::glaive(),
            OffHand::Free,
        );
        assert_eq!(WeaponRange::Melee(Feet(10)), equip2.get_primary_weapon().get_range());

        let equip3 = Equipment::new(
            Armor::padded(),
            Weapon::shortbow(),
            OffHand::Free,
        );
        assert_eq!(WeaponRange::Ranged(Feet(80), Feet(320)), equip3.get_primary_weapon().get_range());
    }

    #[test]
    fn versatile_test() {
        let equip1 = Equipment::new(
            Armor::splint(),
            Weapon::longsword(),
            OffHand::Shield(ACSource::shield())
        );
        assert_eq!(&DamageDice::D8, equip1.get_primary_dmg_dice());

        let equip2 = Equipment::new(
            Armor::breastplate(),
            Weapon::longsword(),
            OffHand::Free
        );
        assert_eq!(&DamageDice::D10, equip2.get_primary_dmg_dice());
    }
}
