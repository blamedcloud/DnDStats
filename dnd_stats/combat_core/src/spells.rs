use std::collections::HashMap;
use crate::ability_scores::ForceSave;
use crate::attack::basic_attack::BasicAttack;
use crate::damage::BasicDamageManager;

// yes I could just use numbers, no I don't feel like it
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone)]
pub enum SpellSlot {
    Cantrip,
    First,
    Second,
    Third,
    Fourth,
    Fifth,
    Sixth,
    Seventh,
    Eighth,
    Ninth,
}

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone)]
pub enum SpellName {
    FireBolt,
    Fireball,
}

#[derive(Debug, Clone)]
pub enum SpellEffect {
    SpellAttack(BasicAttack),
    SaveDamage(SaveDmgSpell),
    ApplyCondition
}

#[derive(Debug, Clone)]
pub struct SaveDmgSpell {
    pub save: ForceSave,
    pub dmg: BasicDamageManager,
    pub half_dmg: bool,
}

impl SaveDmgSpell {
    pub fn new(save: ForceSave, dmg: BasicDamageManager, half_dmg: bool) -> Self {
        Self {
            save,
            dmg,
            half_dmg
        }
    }
}

#[derive(Debug, Clone)]
pub struct Spell {
    pub slot: SpellSlot,
    pub effect: SpellEffect,
}

impl Spell {
    pub fn new(slot: SpellSlot, effect: SpellEffect) -> Self {
        Self {
            slot,
            effect,
        }
    }
}

pub type SpellManager = HashMap<SpellName, Spell>;
