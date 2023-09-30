use serde::{Deserialize, Serialize};
use crate::participant::{ParticipantId, TeamMember};
use crate::strategy::{Strategy, StrategyBuilder};
use crate::strategy::action_surge_str::ActionSurgeStrBuilder;
use crate::strategy::basic_atk_str::BasicAtkStrBuilder;
use crate::strategy::basic_strategies::{DoNothingBuilder, RemoveCondBuilder};
use crate::strategy::dual_wield_str::DualWieldStrBuilder;
use crate::strategy::favored_foe_str::FavoredFoeStrBldr;
use crate::strategy::fireball_str::FireBallStrBuilder;
use crate::strategy::firebolt_str::FireBoltStrBuilder;
use crate::strategy::greater_invis_str::GreaterInvisStrBuilder;
use crate::strategy::gwm_str::GWMStrBldr;
use crate::strategy::haste_str::HasteStrBuilder;
use crate::strategy::linear_str::LinearStrategy;
use crate::strategy::planar_warrior_str::PlanarWarriorStrBldr;
use crate::strategy::second_wind_str::SecondWindStrBuilder;
use crate::strategy::sharp_shooter_str::SharpShooterStrBldr;
use crate::strategy::shield_master_str::ShieldMasterStrBuilder;
use crate::strategy::sneak_atk_str::SneakAttackStrBuilder;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StrategyBuilderDescription {
    Basic(StrategyBuilderName),
    List(Vec<StrategyBuilderName>),
}

impl StrategyBuilder for StrategyBuilderDescription {
    fn build_strategy<'pm>(&self, participants: &'pm Vec<TeamMember>, me: ParticipantId) -> Box<dyn Strategy + 'pm> {
        match self {
            StrategyBuilderDescription::Basic(sbn) => sbn.build_strategy(participants, me),
            StrategyBuilderDescription::List(sbns) => {
                let mut strategies = Vec::with_capacity(sbns.len());
                for sbn in sbns.iter() {
                    strategies.push(sbn.build_strategy(participants, me));
                }
                Box::new(LinearStrategy::new(strategies))
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StrategyBuilderName {
    ActionSurgeSB,
    BasicAtkSB,
    DoNothingSB,
    RemoveCondSB,
    DualWieldSB,
    FavoredFoeSB,
    FireBallSB,
    FireBoltSB,
    GreaterInvisSB,
    GreatWeaponMasterSB(bool),
    SharpShooterSB(bool),
    HasteSB,
    PlanarWarriorSB,
    SecondWindSB,
    ShieldMasterSB,
    SneakAttackSB(bool),
}

impl StrategyBuilder for StrategyBuilderName {
    fn build_strategy<'pm>(&self, participants: &'pm Vec<TeamMember>, me: ParticipantId) -> Box<dyn Strategy + 'pm> {
        match self {
            StrategyBuilderName::ActionSurgeSB => ActionSurgeStrBuilder.build_strategy(participants, me),
            StrategyBuilderName::BasicAtkSB => BasicAtkStrBuilder.build_strategy(participants, me),
            StrategyBuilderName::DoNothingSB => DoNothingBuilder.build_strategy(participants, me),
            StrategyBuilderName::RemoveCondSB => RemoveCondBuilder.build_strategy(participants, me),
            StrategyBuilderName::DualWieldSB => DualWieldStrBuilder.build_strategy(participants, me),
            StrategyBuilderName::FavoredFoeSB => FavoredFoeStrBldr.build_strategy(participants, me),
            StrategyBuilderName::FireBallSB => FireBallStrBuilder.build_strategy(participants, me),
            StrategyBuilderName::FireBoltSB => FireBoltStrBuilder.build_strategy(participants, me),
            StrategyBuilderName::GreaterInvisSB => GreaterInvisStrBuilder.build_strategy(participants, me),
            StrategyBuilderName::GreatWeaponMasterSB(use_gwm) => GWMStrBldr::new(*use_gwm).build_strategy(participants, me),
            StrategyBuilderName::SharpShooterSB(use_ss) => SharpShooterStrBldr::new(*use_ss).build_strategy(participants, me),
            StrategyBuilderName::HasteSB => HasteStrBuilder.build_strategy(participants, me),
            StrategyBuilderName::PlanarWarriorSB => PlanarWarriorStrBldr.build_strategy(participants, me),
            StrategyBuilderName::SecondWindSB => SecondWindStrBuilder.build_strategy(participants, me),
            StrategyBuilderName::ShieldMasterSB => ShieldMasterStrBuilder.build_strategy(participants, me),
            StrategyBuilderName::SneakAttackSB(greedy) => SneakAttackStrBuilder::new(*greedy).build_strategy(participants, me),
        }
    }
}
