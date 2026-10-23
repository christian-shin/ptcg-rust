//! Emma (CRI 77, Supporter): your opponent reveals their hand, and you draw a card for each Pokémon you find there.
//!
//! The hand is revealed first (a Reveal routine, as Eri), then one Draw event of as many cards as the revealed hand holds
//! Pokémon (0 when none: the Supporter was still playable, the reveal is its effect). With no cards in the opponent's hand it
//! can't be played (id1492: revealing an empty hand isn't revealing it).
use crate::spec::prelude::*;

const OPP_HAND: ZoneRef = ZoneRef(Who::Opp, Zone::Hand);

pub static SPEC: CardSpec = CardSpec {
    class: "Emma",
    play: Some(PlaySpec {
        kind: PlayKind::Supporter,
        needs: &[],
        steps: &[
            Step::new(Op::Reveal(RevealSpec { cards: RevealWhat::Zone(OPP_HAND), by: Who::Opp, to: Who::Me, when_empty: true })),
            Step::new(Op::Draw(DrawSpec { who: Who::Me, amount: DrawAmount::Count(Num::CardCount(OPP_HAND, Pred::Pokemon)) })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
