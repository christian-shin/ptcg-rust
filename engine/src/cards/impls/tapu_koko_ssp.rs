//! Tapu Koko (SSP 65): Summon Lightning - search your deck for up to 2 [L] Pokémon, reveal them, and put them into your hand. Then,
//! shuffle your deck. Prize Count - 90+; if you have more Prize cards remaining than your opponent, this attack does 90 more damage.
//!
//! Summon Lightning: one search (0 to 2 [L] Pokémon, revealed) and the shuffle (APR E-19); an empty deck can't be searched (id840).
//! Prize Count compares the Prize cards each player has left before the damage.
use crate::spec::prelude::*;

const DECK: ZoneRef = ZoneRef(Who::Me, Zone::Deck);

pub static SPEC: CardSpec = CardSpec {
    class: "TapuKokoSSP",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[Step::after_damage(Op::If(IfSpec {
                cond: Cond::Nonempty(DECK, Pred::Any),
                yes: &[
                    Step::new(Op::Search(SearchSpec {
                        pick: PickSpec { predicate: Pred::All(&[Pred::Pokemon, Pred::PokemonType(ct::LIGHTNING)]), bounds: Bounds { min: Num::Lit(0), max: Num::Lit(2) }, ..PickSpec::DEFAULT },
                        destination: SearchDestination::Hand { reveal: true },
                        cancel: false,
                    })),
                    Step::new(Op::Shuffle(ShuffleSpec { zone: DECK, wait: true })),
                ],
                no: &[],
            }))],
        },
        AttackSpec { index: 1, steps: &[Step::before_damage(more_damage_if(90, Cond::Cmp(Num::PrizesLeft(Who::Me), CmpOp::Gt, Num::PrizesLeft(Who::Opp))))] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
