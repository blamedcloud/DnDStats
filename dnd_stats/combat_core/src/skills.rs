use num::{FromPrimitive, Rational64};

use rand_var::map_rand_var::MapRandVar;
use rand_var::num_rand_var::NumRandVar;
use rand_var::rand_var::prob_type::RVProb;
use rand_var::vec_rand_var::{VRV64, VecRandVar};

use crate::{BinaryOutcome, D20RollType, D20Type};
use crate::ability_scores::{Ability, AbilityScores};
use crate::participant::Participant;

// For the time being, only include skills useful in combat,
// and by "useful" I mean those with concrete rules. Nothing
// DM dependant here, folks.
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone)]
pub enum SkillName {
    Acrobatics,
    Athletics,
    Perception,
    Stealth,
}

impl SkillName {
    pub fn get_ability(&self) -> Ability {
        match self {
            SkillName::Acrobatics => Ability::DEX,
            SkillName::Athletics => Ability::STR,
            SkillName::Perception => Ability::WIS,
            SkillName::Stealth => Ability::DEX,
        }
    }
}

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone)]
pub enum ProfType {
    Normal,
    HalfProf,
    Proficient,
    Expert,
}

impl ProfType {
    pub fn get_bonus(&self, prof: isize) -> isize {
        match self {
            ProfType::Normal => 0,
            ProfType::HalfProf => prof / 2,
            ProfType::Proficient => prof,
            ProfType::Expert => prof * 2,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct SkillCheck {
    pub override_ability: Option<Ability>,
    pub prof_type: ProfType,
    pub default_roll_type: D20RollType,
    pub d20_type: D20Type,
    pub passive_bonus: isize,
}

impl SkillCheck {
    pub fn new() -> Self {
        Self {
            override_ability: None,
            prof_type: ProfType::Normal,
            default_roll_type: D20RollType::Normal,
            d20_type: D20Type::D20,
            passive_bonus: 0,
        }
    }

    pub fn get_default_rv<P: RVProb>(&self) -> VecRandVar<P> {
        self.default_roll_type.get_rv(&self.d20_type)
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct SkillManager {
    pub acrobatics: SkillCheck,
    pub athletics: SkillCheck,
    pub perception: SkillCheck,
    pub stealth: SkillCheck,
}

impl SkillManager {
    pub fn new() -> Self {
        Self {
            acrobatics: SkillCheck::new(),
            athletics: SkillCheck::new(),
            perception: SkillCheck::new(),
            stealth: SkillCheck::new(),
        }
    }

    pub fn get_skill_check(&self, skill: SkillName) -> &SkillCheck {
        match skill {
            SkillName::Acrobatics => &self.acrobatics,
            SkillName::Athletics => &self.athletics,
            SkillName::Perception => &self.perception,
            SkillName::Stealth => &self.stealth,
        }
    }

    pub fn get_skill_check_mut(&mut self, skill: SkillName) -> &mut SkillCheck {
        match skill {
            SkillName::Acrobatics => &mut self.acrobatics,
            SkillName::Athletics => &mut self.athletics,
            SkillName::Perception => &mut self.perception,
            SkillName::Stealth => &mut self.stealth,
        }
    }

    pub fn get_skill_rv<P: RVProb>(&self, skill: SkillName, ability_scores: &AbilityScores, prof: isize) -> VecRandVar<P> {
        let skill_check = self.get_skill_check(skill);
        let ability = skill_check.override_ability.unwrap_or(skill.get_ability());
        let mut check_bonus = ability_scores.get_score(&ability).get_mod() as isize;
        check_bonus += skill_check.prof_type.get_bonus(prof);

        let d20 = skill_check.get_default_rv();
        let rv = d20.add_const(check_bonus);
        rv
    }

    pub fn meets_dc<P: RVProb>(&self, skill: SkillName, ability_scores: &AbilityScores, prof: isize, dc: isize) -> MapRandVar<BinaryOutcome, P> {
        let skill_rv = self.get_skill_rv(skill, ability_scores, prof);
        skill_rv.into_mrv().map_keys(|check| {
            if check >= dc {
                BinaryOutcome::Pass
            } else {
                BinaryOutcome::Fail
            }
        })
    }

    pub fn choose_grapple_defense(&self, ability_scores: &AbilityScores, prof: isize) -> SkillName {
        let acro = &self.acrobatics;
        let acro_abil = acro.override_ability.unwrap_or(SkillName::Acrobatics.get_ability());
        let mut acro_bonus = ability_scores.get_score(&acro_abil).get_mod() as isize;
        acro_bonus += acro.prof_type.get_bonus(prof);
        let acro_rv: VRV64 = acro.get_default_rv();
        let acro_ev = acro_rv.expected_value() + Rational64::from_isize(acro_bonus).unwrap();

        let athl = &self.athletics;
        let athl_abil = athl.override_ability.unwrap_or(SkillName::Athletics.get_ability());
        let mut athl_bonus = ability_scores.get_score(&athl_abil).get_mod() as isize;
        athl_bonus += athl.prof_type.get_bonus(prof);
        let athl_rv: VRV64 = athl.get_default_rv();
        let athl_ev = athl_rv.expected_value() + Rational64::from_isize(athl_bonus).unwrap();

        if acro_ev > athl_ev {
            SkillName::Acrobatics
        } else {
            SkillName::Athletics
        }
    }

    pub fn get_grapple_defense_rv<P: RVProb>(&self, ability_scores: &AbilityScores, prof: isize) -> VecRandVar<P> {
        let skill = self.choose_grapple_defense(ability_scores, prof);
        self.get_skill_rv(skill, ability_scores, prof)
    }
}

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone)]
pub enum ContestResult {
    InitiatorWins,
    DefenderWins,
}

#[derive(Debug, Clone)]
pub struct SkillContest<P: RVProb> {
    pub result: MapRandVar<ContestResult, P>,
}

impl<P: RVProb> SkillContest<P> {
    pub fn new(initiator_rv: &VecRandVar<P>, defender_rv: &VecRandVar<P>) -> Self {
        let result = initiator_rv.minus_rv(defender_rv)
            .into_mrv()
            .map_keys(|result| {
            if result >= 0 {
                ContestResult::InitiatorWins
            } else {
                ContestResult::DefenderWins
            }
        });
        Self {
            result
        }
    }

    pub fn build(initiator: &Box<dyn Participant>, defender: &Box<dyn Participant>, initiator_skill: SkillName, defender_skill: SkillName) -> Self {
        let i_sm = initiator.get_skill_manager();
        let d_sm = defender.get_skill_manager();
        let i_rv = i_sm.get_skill_rv(initiator_skill, initiator.get_ability_scores(), initiator.get_prof());
        let d_rv = d_sm.get_skill_rv(defender_skill, defender.get_ability_scores(), defender.get_prof());
        SkillContest::new(&i_rv, &d_rv)
    }

    pub fn grapple_like(grappler: &Box<dyn Participant>, target: &Box<dyn Participant>, grappler_is_initiator: bool) -> Self {
        let g_sm = grappler.get_skill_manager();
        let t_sm = target.get_skill_manager();

        let g_rv = g_sm.get_skill_rv(SkillName::Athletics, grappler.get_ability_scores(), grappler.get_prof());
        let t_rv = t_sm.get_grapple_defense_rv(target.get_ability_scores(), target.get_prof());
        if grappler_is_initiator {
            SkillContest::new(&g_rv, &t_rv)
        } else {
            SkillContest::new(&t_rv, &g_rv)
        }
    }
}

#[cfg(test)]
mod tests {
    use num::{One, Rational64};
    use rand_var::num_rand_var::NumRandVar;

    use rand_var::vec_rand_var::VRV64;
    use rand_var::rand_var::RandVar;

    use crate::{D20RollType, D20Type};
    use crate::ability_scores::AbilityScores;
    use crate::skills::{ContestResult, ProfType, SkillContest, SkillManager, SkillName};

    pub fn get_dex_based() -> AbilityScores {
        AbilityScores::new(12,16,16,8,13,10)
    }

    pub fn get_str_based() -> AbilityScores {
        AbilityScores::new(16,12,16,8,13,10)
    }

    #[test]
    fn basic_test() {
        let mut default_sm = SkillManager::new();
        let acro = &default_sm.acrobatics;
        assert_eq!(D20Type::D20, acro.d20_type);
        assert_eq!(D20RollType::Normal, acro.default_roll_type);
        assert_eq!(ProfType::Normal, acro.prof_type);
        let rv: VRV64 = default_sm.get_skill_rv(SkillName::Acrobatics, &get_dex_based(), 2);
        assert_eq!(4, rv.lower_bound());
        assert_eq!(23, rv.upper_bound());

        default_sm.acrobatics.prof_type = ProfType::Proficient;
        let rv: VRV64 = default_sm.get_skill_rv(SkillName::Acrobatics, &get_dex_based(), 2);
        assert_eq!(6, rv.lower_bound());
        assert_eq!(25, rv.upper_bound());
    }

    #[test]
    fn grapple_test_low_level() {
        let rogue_as = get_dex_based();
        let mut rogue_sm = SkillManager::new();
        rogue_sm.acrobatics.prof_type = ProfType::Expert;

        let barb_as = get_str_based();
        let mut barb_sm = SkillManager::new();
        barb_sm.athletics.prof_type = ProfType::Proficient;
        barb_sm.athletics.default_roll_type = D20RollType::Advantage;

        let prof = 2;

        let rogue_rv = rogue_sm.get_grapple_defense_rv(&rogue_as, prof);
        assert_eq!(8, rogue_rv.lower_bound());
        assert_eq!(27, rogue_rv.upper_bound());
        assert_eq!(Rational64::new(35, 2), rogue_rv.expected_value());

        let barb_rv = barb_sm.get_skill_rv(SkillName::Athletics, &barb_as, prof);
        assert_eq!(6, barb_rv.lower_bound());
        assert_eq!(25, barb_rv.upper_bound());
        assert_eq!(Rational64::new(753, 40), barb_rv.expected_value());

        let contest = SkillContest::new(&barb_rv, &rogue_rv);
        let barb_wins = Rational64::new(4731,8000);
        assert_eq!(barb_wins, contest.result.pdf(ContestResult::InitiatorWins));
        let rogue_wins = Rational64::one() - barb_wins;
        assert_eq!(rogue_wins, contest.result.pdf(ContestResult::DefenderWins));
    }

    #[test]
    fn grapple_test_high_level() {
        let mut rogue_as = get_dex_based();
        rogue_as.dexterity.set_score(20);
        let mut rogue_sm = SkillManager::new();
        rogue_sm.acrobatics.prof_type = ProfType::Expert;
        rogue_sm.acrobatics.d20_type = D20Type::D20m10;

        let mut barb_as = get_str_based();
        barb_as.strength.set_score(20);
        let mut barb_sm = SkillManager::new();
        barb_sm.athletics.prof_type = ProfType::Proficient;
        barb_sm.athletics.default_roll_type = D20RollType::Advantage;

        let prof = 4;

        let rogue_rv = rogue_sm.get_grapple_defense_rv(&rogue_as, prof);
        assert_eq!(23, rogue_rv.lower_bound());
        assert_eq!(33, rogue_rv.upper_bound());
        assert_eq!(Rational64::new(103, 4), rogue_rv.expected_value());

        let barb_rv = barb_sm.get_skill_rv(SkillName::Athletics, &barb_as, prof);
        assert_eq!(10, barb_rv.lower_bound());
        assert_eq!(29, barb_rv.upper_bound());
        assert_eq!(Rational64::new(913, 40), barb_rv.expected_value());

        let contest = SkillContest::new(&barb_rv, &rogue_rv);
        let barb_wins = Rational64::new(3059,8000);
        assert_eq!(barb_wins, contest.result.pdf(ContestResult::InitiatorWins));
        let rogue_wins = Rational64::one() - barb_wins;
        assert_eq!(rogue_wins, contest.result.pdf(ContestResult::DefenderWins));
    }
}
