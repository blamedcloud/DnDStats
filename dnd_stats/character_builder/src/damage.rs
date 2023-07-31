
#[derive(Debug, PartialEq)]
pub enum DamageDice {
    D4,
    D6,
    D8,
    D10,
    D12,
    TwoD6,
}

#[derive(Debug, PartialEq)]
pub enum DamageType {
    Acid,
    Bludgeoning,
    Cold,
    Fire,
    Force,
    Lightning,
    Necrotic,
    Piercing,
    Poison,
    Psychic,
    Radiant,
    Slashing,
    Thunder,
}
