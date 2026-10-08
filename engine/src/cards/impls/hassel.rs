//! Hassel (TWM): only if any of your Pokémon were Knocked Out during your
//! opponent's last turn. Look at the top 8 cards of your deck; put up to 3
//! into your hand and shuffle the rest into your deck.
//!
//! Twinleaf: every Hassel copy (in any zone) adds its own HASSEL_MARKER to its
//! owner when that player's Pokémon is Knocked Out during the opponent's
//! turn; the marker check comes after the Supporter moves to the supporter
//! pile. The remaining top cards go back to the deck bottom before the
//! (wait-less) shuffle is answered.
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "Hassel",
    play: Some(PlaySpec {
        kind: PlayKind::Supporter,
        needs: &[Cond::KnockedOutLastTurn { who: Who::Me, by_attack: false, tag: None }],
        steps: &[
            Step::new(Op::Move(MoveSpec { from: ZoneRef(Who::Me, Zone::Deck), to: ZoneRef(Who::Me, Zone::Scratch(0)), cards: CardSel::Top(Num::Lit(8)), ..MoveSpec::DEFAULT })),
            Step::new(Op::Pick(PickSpec { from: ZoneRef(Who::Me, Zone::Scratch(0)), bounds: Bounds { min: Num::If(&Cond::ViaAttack, &Num::Lit(0), &Num::Lit(1)), max: Num::Lit(3) }, into: 1, msg: "CHOOSE_CARD_TO_HAND", ..PickSpec::DEFAULT })),
            Step::new(Op::Move(MoveSpec { from: ZoneRef(Who::Me, Zone::Scratch(0)), to: ZoneRef(Who::Me, Zone::Hand), cards: CardSel::Chosen(1), ..MoveSpec::DEFAULT })),
            Step::new(Op::Move(MoveSpec { from: ZoneRef(Who::Me, Zone::Scratch(0)), to: ZoneRef(Who::Me, Zone::Deck), cards: CardSel::All, ..MoveSpec::DEFAULT })),
            Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck) })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
