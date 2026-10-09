//! Hariyama (MEG 73): Heave-Ho Catcher — once during your turn, when you play
//! this Pokémon from your hand to evolve 1 of your Pokémon, you may switch in 1
//! of your opponent's Benched Pokémon to the Active Spot. Wild Press — 210;
//! this Pokémon also does 70 damage to itself.
//!
//! Rule: the switch-in is a ChangeActive (SwitchIn) by the Ability, done to the Benched Pokémon chosen. A Benched
//! Pokémon with Hide 'n' Sneak can be chosen but isn't switched in; Hide 'n' Sneak on the Active Pokémon doesn't stop
//! it (official JP Q&A, Hariyama MEG 73). Evolving from the hand includes Rare Candy (id285, id1998).
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "Hariyama",
    attacks: &[
        AttackSpec { index: 0, steps: &[
            Step::after_damage(self_damage(70)),
        ] },
    ],
    triggers: &[
        Trigger { origin: RuleSource::Ability, event: Event::On(EventPred::All(&[EventPred::Kind(EventKind::Evolve), EventPred::This(Role::Card), EventPred::Source(RulesZone::Hand)])), steps: &[Step::new(Op::If(IfSpec { cond: Cond::All(&[Cond::Not(&Cond::AbilityBlocked), Cond::AnySlot(SlotSel::Bench(Who::Opp), SlotPred::Any)]), yes: &[Step::new(Op::May(MaySpec { asker: Who::Me, when: Cond::True, msg: "WANT_TO_USE_ABILITY", yes: &[Step::new(Op::Switch(SwitchSpec { change: ActiveChange::SwitchIn, among: SwitchAmong::Bench, msg: "CHOOSE_POKEMON_TO_SWITCH", required: false }))], no: &[] }))], no: &[] }))] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
