//! Hariyama (M1L): Sumo Catcher — when you play this card from your hand to
//! evolve, you may switch 1 of your opponent's Benched Pokémon with their
//! Active Pokémon. Wild Press — 210; this Pokémon does 70 damage to itself.
//!
//! Twinleaf: fires on any EvolveEffect for this card (Rare Candy included);
//! the switch goes through an EffectOfAbilityEffect and happens only if its
//! target survives.
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "Hariyama",
    attacks: &[
        AttackSpec { index: 0, steps: &[
            Step::after_damage(self_damage(70)),
        ] },
    ],
    triggers: &[
        Trigger { origin: RuleSource::Ability, event: Event::On(EventPred::All(&[EventPred::Kind(EventKind::Evolve), EventPred::This(Role::Card), EventPred::Source(RulesZone::Hand)])), steps: &[Step::new(Op::If(IfSpec { cond: Cond::All(&[Cond::Not(&Cond::AbilityBlocked), Cond::AnySlot(SlotSel::Bench(Who::Opp), SlotPred::Any)]), yes: &[Step::new(Op::May(MaySpec { asker: Who::Me, when: Cond::True, msg: "WANT_TO_USE_ABILITY", yes: &[Step::new(Op::Switch(SwitchSpec { side: Who::Opp, chooser: Who::Me, kind: SwitchKind::SilentAbilityEffect, msg: "CHOOSE_POKEMON_TO_SWITCH", required: false }))], no: &[] }))], no: &[] }))] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
