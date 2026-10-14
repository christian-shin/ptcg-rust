//! Flareon ex (PRE 14, Tera): Burning Charge — 130; search your deck for up to 2 Basic Energy cards and attach them to 1 of your
//! Pokémon. Then, shuffle your deck. Carnelian — 280; during your next turn, this Pokémon can't attack. Tera: prevent all
//! damage done to this Pokémon by attacks while it is on the Bench.
//!
//! Burning Charge: with a card in the deck, one search of 0 to 2 Basic Energy cards (a search for a kind: it may take none, JP
//! FAQ audit), the cards attached to one Pokémon of yours (the Active counts), then the shuffle after the search (APR E-19).
//! Carnelian arms the `CANT_ATTACK` lock on this Pokémon until your next turn. Tera is `TERA_RULE`.
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "Flareonex",
    attacks: &[
        AttackSpec { index: 0, steps: &[
            Step::after_damage(Op::If(IfSpec { cond: Cond::Nonempty(ZoneRef(Who::Me, Zone::Deck), Pred::Any), yes: &[Step::new(Op::Search(SearchSpec {
                pick: PickSpec { from: ZoneRef(Who::Me, Zone::Deck), predicate: Pred::BasicEnergy, bounds: Bounds { min: Num::Lit(0), max: Num::Lit(2) }, ..PickSpec::DEFAULT },
                destination: SearchDestination::AttachToPicked,
                cancel: false,
            })), Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true }))], no: &[] })),
        ] },
        AttackSpec { index: 1, steps: &[
            Step::after_damage(Op::Arm(ArmSpec { what: Lasting::Lock(LastingLockSpec::on_this_pokemon(&CANT_ATTACK, LockUntil::YourNextTurn)) })),
        ] },
    ],
    passives: &[
        Passive { origin: RuleSource::CardRule, modifier: Modifier::Prevent(TERA_RULE) },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
