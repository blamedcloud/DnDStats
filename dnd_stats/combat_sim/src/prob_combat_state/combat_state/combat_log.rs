use std::rc::Rc;
use crate::prob_combat_state::combat_state::combat_log::combat_event::CombatEvent;

pub mod combat_event;

#[derive(Debug, Clone)]
pub struct CombatLog {
    parent: Option<CombatLogRef>,
    events: Vec<CombatEvent>,
}
type CombatLogRef = Rc<CombatLog>;

impl CombatLog {
    pub fn new() -> Self {
        Self {
            parent: None,
            events: Vec::new(),
        }
    }

    pub fn push(&mut self, ce: CombatEvent) {
        self.events.push(ce);
    }

    pub fn get_local_events(&self) -> &Vec<CombatEvent> {
        &self.events
    }

    pub fn get_all_events(&self) -> Vec<CombatEvent> {
        let mut all_events = Vec::new();
        if self.parent.is_some() {
            all_events = self.parent.as_ref().unwrap().get_all_events();
        }
        all_events.extend(self.events.iter());
        all_events
    }

    pub fn has_parent(&self) -> bool {
        self.parent.is_some()
    }

    pub fn get_parent(&self) -> &CombatLogRef {
        self.parent.as_ref().unwrap()
    }

    pub fn get_last_event(&self) -> Option<CombatEvent> {
        if self.events.len() > 0 {
            self.events.last().copied()
        } else {
            if self.has_parent() {
                self.get_parent().get_last_event()
            } else {
                None
            }
        }
    }

    pub fn into_child(self) -> Self {
        Self {
            parent: Some(Rc::new(self)),
            events: Vec::new(),
        }
    }
}
