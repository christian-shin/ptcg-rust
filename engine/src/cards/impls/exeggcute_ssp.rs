//! Exeggcute (SSP 1): Precocious Evolution (usable on the first turn) — search your deck for a card that evolves from this
//! Pokémon and put it onto this Pokémon to evolve it. Then, shuffle your deck.
//!
//! "Usable on the first turn" ("If you go first, you can use this attack on your first turn") is
//! `AttackFlags { first_turn: Attack(0) }` on Precocious Evolution, read by the UseAttack's first-turn check.
//!
//! An empty deck can't be searched (id840): nothing happens, no shuffle. The evolving is an Evolve event from the deck (not
//! played from the hand), caused by the attack; the choice can be declined (a search for a kind).
use crate::spec::prelude::*;

const DECK: ZoneRef = ZoneRef(Who::Me, Zone::Deck);

pub static SPEC: CardSpec = CardSpec {
    class: "Exeggcute@SSP",
    passives: &[Passive { origin: RuleSource::CardRule, modifier: Modifier::AttackFlags(AttackFlagsSpec { first_turn: Some(FirstTurnAttack::Attack(0)), shred: None }) }],
    // Precocious Evolution: search your deck for a card that evolves from this Pokémon and put it
    // onto this Pokémon to evolve it, then shuffle.
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::after_damage(Op::If(IfSpec {
            cond: Cond::Nonempty(DECK, Pred::Any),
            yes: &[
                Step::new(Op::Pick(PickSpec {
                    from: DECK,
                    predicate: Pred::All(&[Pred::Pokemon, Pred::EvolvesFromActive]),
                    bounds: Bounds { min: Num::Lit(0), max: Num::Lit(1) },
                    into: 0,
                    purpose: Some(PromptPurpose::EvolutionCard),
                    ..PickSpec::DEFAULT
                })),
                Step::new(Op::Evolve(EvolveSpec { how: EvolveHow::PutOnto { slot: MY_ACTIVE, card: 0 } })),
                Step::new(Op::Shuffle(ShuffleSpec { zone: DECK, wait: true })),
            ],
            no: &[],
        }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
