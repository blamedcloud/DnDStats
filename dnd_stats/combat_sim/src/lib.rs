use character_builder::combat::{ActionName, ActionType};
use character_builder::resources::ResourceName;
use crate::combat_log::{CombatEvent, CombatLog, CombatTiming};
use crate::participant::{Participant, ParticipantId, TeamMember};
use crate::strategy::{StrategicOption, Strategy};

pub mod combat_log;
pub mod participant;
pub mod strategy;
pub mod target_dummy;


pub struct EncounterManager {
    participants: Vec<TeamMember>, // in order of initiative
    strategies: Vec<Box<dyn Strategy>>,
    round_num: u8,
    combat_log: CombatLog,
}

impl EncounterManager {
    pub fn new() -> Self {
        EncounterManager {
            participants: Vec::new(),
            strategies: Vec::new(),
            round_num: 0,
            combat_log: CombatLog::new(),
        }
    }

    pub fn add_participant(&mut self, tm: TeamMember, str: Box<dyn Strategy>) {
        self.participants.push(tm);
        self.strategies.push(str);
    }

    pub fn get_participant(&self, pid: ParticipantId) -> &Box<dyn Participant> {
        &self.participants.get(pid.0).unwrap().participant
    }
    fn get_participant_mut(&mut self, pid: ParticipantId) -> &mut Box<dyn Participant> {
        &mut self.participants.get_mut(pid.0).unwrap().participant
    }

    pub fn get_strategy(&self, pid: ParticipantId) -> &Box<dyn Strategy> {
        &self.strategies.get(pid.0).unwrap()
    }

    pub fn register_event(&mut self, event: CombatEvent) {
        self.combat_log.push(event);

        if let CombatEvent::Timing(ct) = event {
            for i in 0..self.participants.len() {
                let pid = ParticipantId(i);
                let rt = ct.get_refresh_timing(pid);
                self.get_participant_mut(pid).get_resource_manager_mut().handle_timing(rt);
            }
        }

        // TODO: setup triggers for participants for non-timing events
    }

    pub fn simulate_n_rounds(&mut self, n: u8) {
        while self.round_num < n {
            self.register_event(CombatEvent::Timing(CombatTiming::BeginRound));
            self.simulate_round();
            self.register_event(CombatEvent::Timing(CombatTiming::EndRound));
            self.round_num += 1;
        }
    }

    fn simulate_round(&mut self) {
        for i in 0..self.participants.len() {
            let pid = ParticipantId(i);
            self.register_event(CombatEvent::Timing(CombatTiming::BeginTurn(pid)));
            self.simulate_turn(pid);
            self.register_event(CombatEvent::Timing(CombatTiming::EndTurn(pid)));
        }
    }

    fn simulate_turn(&mut self, pid: ParticipantId) {
        loop {
            let strategy = self.get_strategy(pid);
            if let Some(so) = strategy.get_action(&self.combat_log, &self.participants, pid) {
                if self.possible_action(pid, so) {
                    self.handle_action(pid, so);
                } else {
                    return;
                }
            } else {
                return;
            }
        }
    }

    fn possible_action(&self, pid: ParticipantId, so: StrategicOption) -> bool {
        let an = so.action_name;
        let participant = self.get_participant(pid);
        let has_action = participant.get_action_manager().contains_key(&an);
        let mut valid_target = true;
        let mut has_resources = true;
        if has_action {
            let co = participant.get_action_manager().get(&an).unwrap();
            if co.req_target && so.target.is_none() {
                valid_target = false;
            }
        }
        if has_action && valid_target {
            let rm = participant.get_resource_manager();
            if rm.has_resource(ResourceName::AN(an)) && rm.get_current(ResourceName::AN(an)) == 0 {
                has_resources = false;
            } else {
                let co = participant.get_action_manager().get(&an).unwrap();
                let at = co.action_type;
                if rm.has_resource(ResourceName::AT(at)) && rm.get_current(ResourceName::AT(at)) == 0 {
                    has_resources = false;
                }
            }
        }
        has_action && valid_target && has_resources
    }

    fn spend_resources(&mut self, pid: ParticipantId, an: ActionName, at: ActionType) {
        let participant = self.get_participant_mut(pid);
        let rm = participant.get_resource_manager_mut();
        if rm.has_resource(ResourceName::AN(an)) {
            rm.spend(ResourceName::AN(an));
        }
        if rm.has_resource(ResourceName::AT(at)) {
            rm.spend(ResourceName::AT(at));
        }
    }

    fn handle_action(&mut self, pid: ParticipantId, so: StrategicOption) {
        let an = so.action_name;
        let participant = self.get_participant_mut(pid);
        let co = participant.get_action_manager().get(&an).unwrap();
        let at = co.action_type;
        self.spend_resources(pid, an, at);
        todo!()
    }

}

#[cfg(test)]
mod tests {
    use character_builder::ability_scores::AbilityScores;
    use character_builder::Character;
    use character_builder::classes::ClassName;
    use character_builder::equipment::{Armor, Equipment, OffHand, Weapon};
    use character_builder::feature::fighting_style::{FightingStyle, FightingStyles};
    use crate::EncounterManager;
    use crate::participant::{Team, TeamMember};
    use crate::strategy::DoNothing;
    use crate::target_dummy::TargetDummy;

    pub fn get_str_based() -> AbilityScores {
        AbilityScores::new(16,12,16,8,13,10)
    }

    pub fn get_test_fighter_lvl0() -> Character {
        let name = String::from("FighterMan");
        let ability_scores = get_str_based();
        let equipment = Equipment::new(
            Armor::chain_mail(),
            Weapon::greatsword(),
            OffHand::Free,
        );
        Character::new(name, ability_scores, equipment)
    }

    #[test]
    fn lvl1fighter_vs_dummy() {
        let mut fighter = get_test_fighter_lvl0();
        fighter.level_up(ClassName::Fighter, vec!(Box::new(FightingStyle(FightingStyles::GreatWeaponFighting)))).unwrap();
        let dummy = TargetDummy::new(isize::MAX, 14);

        let mut em = EncounterManager::new();
        em.add_participant(TeamMember::new(Team::Players, Box::new(fighter)), Box::new(DoNothing));
        em.add_participant(TeamMember::new(Team::Enemies, Box::new(dummy)), Box::new(DoNothing));
    }
}
