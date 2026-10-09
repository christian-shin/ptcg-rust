//! Hop's Dubwool (JTG): Defiant Horn — when you play this Pokémon from your
//! hand to evolve 1 of your Pokémon during your turn, you may switch in 1
//! of your opponent's Benched Pokémon to the Active Spot. Headbutt — 80.
//!
//! Rule: the Ability is offered only when it isn't blocked and the opponent
//! has a Benched Pokémon. The switch-in is a ChangeActive (SwitchIn, APR C-05)
//! by the Ability, done to the Benched Pokémon chosen, the same rule as
//! Hariyama's Heave-Ho Catcher: a Benched Pokémon with Hide 'n' Sneak can be
//! chosen but isn't switched in, and Hide 'n' Sneak on the Active Pokémon
//! doesn't stop it (official JP Q&A, Hariyama MEG 73). Evolving from the hand
//! includes Rare Candy (id285, id1998).
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "HopsDubwool",
    triggers: &[
        Trigger { origin: RuleSource::Ability, event: Event::On(EventPred::All(&[EventPred::Kind(EventKind::Evolve), EventPred::This(Role::Card), EventPred::Source(RulesZone::Hand)])), steps: &[Step::new(Op::If(IfSpec { cond: Cond::All(&[Cond::Not(&Cond::AbilityBlocked), Cond::AnySlot(SlotSel::Bench(Who::Opp), SlotPred::Any)]), yes: &[Step::new(Op::May(MaySpec { asker: Who::Me, when: Cond::True, msg: "WANT_TO_USE_ABILITY", yes: &[Step::new(Op::Switch(SwitchSpec { change: ActiveChange::SwitchIn, among: SwitchAmong::Bench, msg: "CHOOSE_POKEMON_TO_SWITCH", required: false }))], no: &[] }))], no: &[] }))] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
