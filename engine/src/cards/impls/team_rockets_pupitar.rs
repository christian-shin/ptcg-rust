//! Team Rocket's Pupitar (DRI 95): Explosive Ascension - 30; search your deck for a card that evolves from this Pokémon and
//! put it onto this Pokémon to evolve it. Then, shuffle your deck.
//!
//! Dwebble DRI's Ascension for a card of any Stage: a Pick over the deck of a Pokémon that evolves from the Pokémon using
//! the attack (`Pred::EvolvesFromActive`), an Evolve event from the deck caused by the attack (an effect-path evolution: no
//! first-turn or entered-this-turn limit, APR C-12), then the shuffle. Nothing happens with an empty deck (id840); the
//! choice can be declined.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "TeamRocketsPupitar",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::after_damage(Op::If(IfSpec {
            cond: Cond::Nonempty(ZoneRef(Who::Me, Zone::Deck), Pred::Any),
            yes: &[
                Step::new(Op::Pick(PickSpec {
                    from: ZoneRef(Who::Me, Zone::Deck),
                    predicate: Pred::All(&[Pred::Pokemon, Pred::EvolvesFromActive]),
                    bounds: Bounds { min: Num::Lit(1), max: Num::Lit(1) },
                    into: 0,
                    cancel: true,
                    ..PickSpec::DEFAULT
                })),
                Step::new(Op::Evolve(EvolveSpec { how: EvolveHow::PutOnto { slot: MY_ACTIVE, card: 0 } })),
                Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true })),
            ],
            no: &[],
        }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
