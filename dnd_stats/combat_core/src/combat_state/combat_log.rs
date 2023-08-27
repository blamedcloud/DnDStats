use std::rc::Rc;
use crate::combat_event::CombatEvent;
use crate::transposition::Transposition;

#[derive(Debug, Clone)]
pub enum CombatLogParent {
    Empty,
    Single(CombatLogRef),
    Many(Vec<CombatLogRef>),
}
type CombatLogRef = Rc<CombatLog>;

impl CombatLogParent {
    pub fn is_empty(&self) -> bool {
        if let CombatLogParent::Empty = self {
            true
        } else {
            false
        }
    }

    pub fn is_present(&self) -> bool {
        !self.is_empty()
    }

    pub fn is_single(&self) -> bool {
        if let CombatLogParent::Single(_) = self {
            true
        } else {
            false
        }
    }

    pub fn is_many(&self) -> bool {
        if let CombatLogParent::Many(_) = self {
            true
        } else {
            false
        }
    }
}

// TODO: add an alt-history ~ Option<Vec<Vec<CombatEvent>>> instead of multiple parents
// I think that would make the transposition code simpler
#[derive(Debug, Clone)]
pub struct CombatLog {
    parent: CombatLogParent,
    events: Vec<CombatEvent>,
}

impl CombatLog {
    pub fn new() -> Self {
        Self {
            parent: CombatLogParent::Empty,
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
        match &self.parent {
            CombatLogParent::Empty => {},
            CombatLogParent::Single(p) => all_events = p.get_all_events(),
            CombatLogParent::Many(vec) => {
                let mut cl_iter = vec.iter();
                all_events = cl_iter.next().unwrap().get_all_events();
                for cl in cl_iter {
                    all_events.extend(cl.get_local_events().iter());
                }
            }
        }
        all_events.extend(self.events.iter());
        all_events
    }

    pub fn has_parent(&self) -> bool {
        self.parent.is_present()
    }

    pub fn get_first_parent(&self) -> Option<&CombatLogRef> {
        match &self.parent {
            CombatLogParent::Empty => None,
            CombatLogParent::Single(p) => Some(p),
            CombatLogParent::Many(vec) => vec.first()
        }
    }

    pub fn get_last_event(&self) -> Option<CombatEvent> {
        if self.events.len() > 0 {
            self.events.last().copied()
        } else {
            match &self.parent {
                CombatLogParent::Empty => None,
                CombatLogParent::Single(cl) => cl.get_last_event(),
                CombatLogParent::Many(cl_vec) => cl_vec.last().and_then(|cl| cl.get_last_event())
            }
        }
    }

    pub fn into_child(self) -> Self {
        Self {
            parent: CombatLogParent::Single(Rc::new(self)),
            events: Vec::new(),
        }
    }
}

impl Transposition for CombatLog {
    fn is_transposition(&self, other: &Self) -> bool {
        if let Some(CombatEvent::Timing(_)) = self.get_last_event() {
            self.get_last_event() == other.get_last_event()
        } else {
            false
        }
    }

    fn merge_left(&mut self, other: Self){
        let left;
        if self.events.len() > 0 {
            left = self.clone().into_child();
        } else {
            left = self.clone();
        }
        let right;
        if other.events.len() > 0 {
            right = other.into_child();
        } else {
            right = other;
        }
        let new_parent = match (left.parent, right.parent) {
            (CombatLogParent::Empty, CombatLogParent::Empty) => CombatLogParent::Empty,
            (CombatLogParent::Empty, CombatLogParent::Single(p)) => CombatLogParent::Single(p),
            (CombatLogParent::Empty, CombatLogParent::Many(v)) => CombatLogParent::Many(v),
            (CombatLogParent::Single(p), CombatLogParent::Empty) => CombatLogParent::Single(p),
            (CombatLogParent::Single(p), CombatLogParent::Single(q)) => {
                let mut v = Vec::with_capacity(2);
                v.push(p);
                v.push(q);
                CombatLogParent::Many(v)
            },
            (CombatLogParent::Single(p), CombatLogParent::Many(mut v)) => {
                v.insert(0, p);
                CombatLogParent::Many(v)
            },
            (CombatLogParent::Many(v), CombatLogParent::Empty) => CombatLogParent::Many(v),
            (CombatLogParent::Many(mut v), CombatLogParent::Single(p)) => {
                v.push(p);
                CombatLogParent::Many(v)
            },
            (CombatLogParent::Many(mut v1), CombatLogParent::Many(v2)) => {
                v1.extend(v2.into_iter());
                CombatLogParent::Many(v1)
            }
        };
        self.parent = new_parent;
        self.events = left.events;
    }
}
