//! Iron Leaves (TWM 19): Recovery Net: choose up to 2 Pokémon from your discard pile, reveal them, and put them into your
//! hand. Avenging Edge: 100+; 60 more if any of your Pokémon were Knocked Out by damage from an attack during your opponent's
//! last turn.
//!
//! Recovery Net: 0 to min(2, Pokémon in the discard pile) are chosen ("up to" in an attack's effect may take 0, id2219, id2301) and
//! go from the discard pile into the hand (a PutIntoHand event, this attack its cause). The official text (data/official_text.json,
//! TWM 19: "Put up to 2 Pokémon from your discard pile into your hand.") has no "reveal them"; the move reveals them (id787).
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "IronLeaves@TWM",
    attacks: &[
        AttackSpec { index: 0, steps: &[
            Step::after_damage(Op::Search(SearchSpec {
                pick: PickSpec { from: ZoneRef(Who::Me, Zone::Discard), predicate: Pred::Pokemon, bounds: Bounds { min: Num::Lit(0), max: Num::Min(&Num::CardCount(ZoneRef(Who::Me, Zone::Discard), Pred::Pokemon), &Num::Lit(2)) }, ..PickSpec::DEFAULT },
                destination: SearchDestination::Hand { reveal: false },
                cancel: false,
            })),
        ] },
        AttackSpec { index: 1, steps: &[
            Step::before_damage(more_damage_if(60, Cond::KnockedOutLastTurn { who: Who::Me, by_attack_damage: true, tag: None })),
        ] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
