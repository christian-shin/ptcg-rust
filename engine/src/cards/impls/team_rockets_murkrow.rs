//! Team Rocket's Murkrow (DRI): Deceit — search your deck for a Supporter,
//! reveal it, put it into your hand, then shuffle. Torment — 30; choose 1 of
//! the opponent's Active Pokémon's attacks; it can't be used next turn.
//!
//! Twinleaf: Deceit does nothing with an empty deck; the search can be
//! cancelled (no reveal); the shuffle is created in the ShowCards callback
//! (fixed in R1-3: directly in the search callback when nothing was taken,
//! so the deck is always shuffled) and has no animation wait. Torment is
//! OPPONENTS_POKEMON_CANNOT_USE_THAT_ATTACK (printed attacks of the current
//! Active; an OpponentPokemonCannotUseAttackEffect when answered).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "TeamRocketsMurkrow",
    attacks: &[
        AttackSpec {
            index: 0,
            // Deceit: search your deck for a Supporter, reveal it, put it into your hand, then shuffle (the search can be cancelled).
            steps: &[Step::after_damage(Op::If(IfSpec {
                cond: Cond::Nonempty(ZoneRef(Who::Me, Zone::Deck), Pred::Any),
                yes: &[
                    Step::new(Op::Search(SearchSpec {
                        pick: PickSpec { from: ZoneRef(Who::Me, Zone::Deck), predicate: Pred::Supporter, bounds: Bounds { min: Num::Lit(0), max: Num::Lit(1) }, ..PickSpec::DEFAULT },
                        destination: SearchDestination::Hand { reveal: true },
                        msg: "",
                        cancel: true,
                        shuffle_first: false,
                    })),
                    Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck) })),
                ],
                no: &[],
            }))],
        },
        // Torment: choose 1 of the opponent's Active Pokémon's attacks; it can't be used next turn.
        AttackSpec { index: 1, steps: &[Step::after_damage(Op::PickAttack(PickAttackSpec { from: Who::Opp }))] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
