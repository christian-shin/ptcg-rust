//! Team Rocket's Murkrow (ASC 126): Deceit - search your deck for a Supporter card, reveal it, and put it into your hand.
//! Then, shuffle your deck. Torment - 30; choose 1 of your opponent's Active Pokémon's attacks; during your opponent's next
//! turn, that Pokémon can't use that attack.
//!
//! Deceit: the Supporter (none may be found) is revealed and put into the hand (a PutIntoHand event, cause: this attack),
//! then the deck is shuffled once (APR E-19); with an empty deck nothing is searched. Torment: the chosen attack of the
//! Defending Pokémon can't be used during the opponent's next turn (`PickAttack`).
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
                    })),
                    Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true })),
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
