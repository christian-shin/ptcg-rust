//! Snorlax (POR 63): Gormandizer — flip a coin until you get tails. Search your deck for an amount of Basic Energy up to the number
//! of heads and attach it to this Pokémon. Then, shuffle your deck. Collapse — 160; this Pokémon is now Asleep.
//!
//! Gormandizer flips after the damage (this attack does none), then searches the deck for 0 to `heads` Basic Energy cards (an Attach
//! event per card, caused by the attack; the search can be declined) onto this Pokémon, then shuffles once (APR E-19). Collapse puts
//! this Pokémon to sleep after the damage (a Special Condition event on itself).
use crate::spec::prelude::*;

const DECK: ZoneRef = ZoneRef(Who::Me, Zone::Deck);

pub static SPEC: CardSpec = CardSpec {
    class: "SnorlaxPOR",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[Step::after_damage(Op::Coin(CoinSpec {
                flips: Flips::UntilTails,
                then: &[
                    Step::new(Op::Attach(AttachSpec {
                        from: DECK,
                        predicate: Pred::BasicEnergy,
                        slots: AttachSlots::ActiveOnly,
                        bounds: Bounds { min: Num::Lit(0), max: Num::Heads },
                        cancel: true,
                        ..AttachSpec::DEFAULT
                    })),
                    Step::new(Op::Shuffle(ShuffleSpec { zone: DECK, wait: true })),
                ],
                ..CoinSpec::DEFAULT
            }))],
        },
        AttackSpec {
            index: 1,
            steps: &[Step::after_damage(Op::Conditions(ConditionsSpec { target: MY_ACTIVE, change: ConditionChange::Add(&[SpecialCondition::Asleep]), when: Cond::True }))],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
