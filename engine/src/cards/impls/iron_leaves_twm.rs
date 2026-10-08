//! Iron Leaves (TWM): Recovery Net — choose up to 2 Pokémon from your discard
//! pile, reveal them, and put them into your hand. Avenging Edge — 100+; 60
//! more if any of your Pokémon were Knocked Out by attack damage during your
//! opponent's last turn.
//!
//! Twinleaf: Recovery Net does nothing with no Pokémon in the discard pile;
//! otherwise a non-cancellable ChooseCardsPrompt (min 0 since phase 4b R7E:
//! "up to 2" in an attack may take 0, rulings 1721/1778; max min(2, count))
//! and MOVE_CARDS to hand, with no reveal prompt.
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "IronLeaves@TWM",
    attacks: &[
        AttackSpec { index: 0, steps: &[
            Step::after_damage(Op::Search(SearchSpec {
                pick: PickSpec { from: ZoneRef(Who::Me, Zone::Discard), predicate: Pred::Pokemon, bounds: Bounds { min: Num::Lit(0), max: Num::Min(&Num::CardCount(ZoneRef(Who::Me, Zone::Discard), Pred::Pokemon), &Num::Lit(2)) }, ..PickSpec::DEFAULT },
                destination: SearchDestination::Hand { reveal: false },
                msg: "",
                cancel: false,
                shuffle_first: false,
            })),
        ] },
        AttackSpec { index: 1, steps: &[
            Step::before_damage(more_damage_if(60, Cond::KnockedOutLastTurn { who: Who::Me, by_attack_damage: true, tag: None })),
        ] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
