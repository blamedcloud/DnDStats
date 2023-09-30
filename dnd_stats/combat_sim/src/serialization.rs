use serde::{Deserialize, Serialize};
use character_builder::{CBError, Character};
use character_builder::serialization::CharacterDescription;
use combat_core::strategy::serialization::StrategyBuilderDescription;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlayerDescription {
    player: CharacterDescription,
    strategy: StrategyBuilderDescription,
}

impl PlayerDescription {
    pub fn new(cd: CharacterDescription, sbd: StrategyBuilderDescription) -> Self {
        Self {
            player: cd,
            strategy: sbd,
        }
    }

    pub fn get_player(&self) -> Result<Character, CBError> {
        self.player.to_character()
    }

    pub fn get_str_bldr(&self) -> &StrategyBuilderDescription {
        &self.strategy
    }
}

#[cfg(test)]
mod tests {
    use character_builder::classes::ClassName;
    use character_builder::equipment::{ArmorName, WeaponName};
    use character_builder::serialization::{CharacterDescription, EquipmentDescription, FeatureName, LevelUp, OffHandDescription, SubClassName};
    use combat_core::strategy::serialization::{StrategyBuilderDescription, StrategyBuilderName};
    use crate::serialization::PlayerDescription;

    #[test]
    fn json_test() {
        let scores = [10,10,10,10,10,10];
        let mut equipment = EquipmentDescription::new(ArmorName::Leather, WeaponName::Rapier, OffHandDescription::Free);
        equipment.set_wmb(1);
        let mut basic_char = CharacterDescription::new(String::from("basic"), scores, equipment);
        basic_char.lvl_up(LevelUp::from(ClassName::Rogue));
        basic_char.lvl_up(LevelUp::from(ClassName::Rogue));
        basic_char.lvl_up(LevelUp::feature(ClassName::Rogue, FeatureName::Subclass(SubClassName::ScoutRogue)));

        let basic_str = StrategyBuilderDescription::List(vec!(StrategyBuilderName::SneakAttackSB(false), StrategyBuilderName::BasicAtkSB));
        let player_ser = PlayerDescription::new(basic_char, basic_str);

        let player_json = serde_json::to_string_pretty(&player_ser).unwrap();
        //println!("Player json = \n{}", player_json);
        let player_copy: PlayerDescription = serde_json::from_str(&player_json).unwrap();
        assert_eq!(player_ser, player_copy);
    }
}
