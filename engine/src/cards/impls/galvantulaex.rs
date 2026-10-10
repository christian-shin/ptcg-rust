//! Galvantula ex (SCR, Tera): Charged Web — 110+, 110 more if your opponent's Active Pokémon is a Pokémon ex or Pokémon V.
//! Fulgurite — 180; discard all Energy from this Pokémon; during your opponent's next turn they can't play Item cards.
//!
//! The V check covers V / VSTAR / VMAX tags (not V-UNION). Fulgurite discards after the damage (so Voltaic Lightning
//! Energy's +20 counts), then arms the Item lock (one ApplyEffect event on the opponent). The Tera rule is `TERA_RULE`:
//! a `Prevent` over `Kind(Damage)` on this Pokémon while it is on the Bench.
use crate::spec::prelude::*;
use crate::types::tag;

pub static SPEC: CardSpec = CardSpec {
    class: "Galvantulaex",
    attacks: &[
        AttackSpec { index: 0, steps: &[
            Step::before_damage(more_damage_if(110, Cond::Slot(SlotExpr::Active(Who::Opp), SlotPred::OneOf(&[SlotPred::Tag(tag::POKEMON_V), SlotPred::Tag(tag::POKEMON_VSTAR), SlotPred::Tag(tag::POKEMON_VMAX), SlotPred::Tag(tag::POKEMON_EX_LOWER)])))),
        ] },
        AttackSpec { index: 1, steps: &[
            Step::after_damage(Op::DiscardEnergy(DiscardEnergySpec { target: SlotTarget::Slot(SlotExpr::Active(Who::Me)), selection: EnergySelection::AllProvided, ..DiscardEnergySpec::DEFAULT })),
            Step::after_damage(Op::Arm(ArmSpec { what: Lasting::OppCannotPlay(&LockDecl::of(&[LockedAction::PlayItem])) })),
        ] },
    ],
    passives: &[
        Passive { origin: RuleSource::CardRule, modifier: Modifier::Prevent(TERA_RULE) }
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
