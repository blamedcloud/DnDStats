use std::collections::HashMap;
use crate::attack::basic_attack::BasicAttack;

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
    SaveDamage,
    ApplyCondition
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
