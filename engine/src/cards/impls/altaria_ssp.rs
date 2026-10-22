//! Altaria (SSP 134): Humming Charge — search your deck for up to 2 Basic Energy cards and attach them to your Pokémon in any way you
//! like. Then, shuffle your deck. Cotton Wings — 100; flip a coin. If heads, during your opponent's next turn, prevent all damage done to
//! this Pokémon by attacks.
//!
//! Humming Charge is one Attach event per Energy from the deck (caused by the attack): 0 to 2 cards, to any of your Pokémon; the
//! search can be declined; the shuffle follows (APR E-19), as Joltik SCR 50. Cotton Wings flips after the damage; on heads one ApplyEffect
//! event arms a lasting `Prevent` over the damage of attacks on this Pokémon through the opponent's next turn (step 6, APR C-16).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "AltariaSSP",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[
                Step::after_damage(Op::Attach(AttachSpec {
                    from: ZoneRef(Who::Me, Zone::Deck),
                    predicate: Pred::BasicEnergy,
                    slots: AttachSlots::BenchActive,
                    bounds: Bounds { min: Num::Lit(0), max: Num::Lit(2) },
                    cancel: true,
                    ..AttachSpec::DEFAULT
                })),
                Step::after_damage(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true })),
            ],
        },
        AttackSpec {
            index: 1,
            steps: &[Step::after_damage(Op::Coin(CoinSpec { heads: &[Step::new(Op::Arm(ArmSpec { what: Lasting::PreventDamage(DamageSource::Any) }))], ..CoinSpec::DEFAULT }))],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
