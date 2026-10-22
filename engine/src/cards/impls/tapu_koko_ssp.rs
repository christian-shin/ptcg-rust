//! Tapu Koko (SSP 65): Summon Lightning - search your deck for up to 2 [L] Pokémon, reveal them, and put them into your hand.
//! Then, shuffle your deck. Prize Count - 90+; if you have more Prize cards remaining than your opponent, this attack does 90
//! more damage.
//!
//! One Search of 0 to 2 [L] Pokémon revealed into the hand, one shuffle after it (APR E-19; Togedemaru PFL 73). Prize Count adds
//! 90 before the damage when you have more Prize cards left than your opponent.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "TapuKokoSSP",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[Step::after_damage(Op::If(IfSpec {
                cond: Cond::Nonempty(ZoneRef(Who::Me, Zone::Deck), Pred::Any),
                yes: &[
                    Step::new(Op::Search(SearchSpec {
                        pick: PickSpec { predicate: Pred::All(&[Pred::Pokemon, Pred::PokemonType(ct::LIGHTNING)]), bounds: Bounds { min: Num::Lit(0), max: Num::Lit(2) }, ..PickSpec::DEFAULT },
                        destination: SearchDestination::Hand { reveal: true },
                        cancel: false,
                    })),
                    Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true })),
                ],
                no: &[],
            }))],
        },
        AttackSpec { index: 1, steps: &[Step::before_damage(more_damage_if(90, Cond::Cmp(Num::PrizesLeft(Who::Me), CmpOp::Gt, Num::PrizesLeft(Who::Opp))))] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
