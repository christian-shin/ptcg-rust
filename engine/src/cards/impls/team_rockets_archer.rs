//! Team Rocket's Archer (DRI): only if any of your Team Rocket's Pokémon were
//! Knocked Out during your opponent's last turn. Each player shuffles their
//! hand into their deck; you draw 5 cards and your opponent draws 3.
//!
//! Twinleaf quirk kept: every Archer copy (any zone, either owner) adds its
//! own ARCHER_MARKER to the player whose Team Rocket's Pokémon is Knocked Out
//! during the other player's turn. Phase 4b: playable with an empty deck (the
//! hand is shuffled into it first), and the draws (opponent 3, then you 5)
//! run after both ShuffleDeckPrompts are answered (they used to run right
//! after the prompts were created, from the unshuffled deck); the opponent's
//! shuffle/draw is skipped if their MoveCardsEffect is prevented.
//!
//! R7C: `rocket_supporter` is not set when used as the effect of an attack (ruling 1727).
use crate::spec::prelude::*;
use crate::types::tag;

pub static SPEC: CardSpec = CardSpec {
    class: "TeamRocketsArcher",
    play: Some(PlaySpec {
        kind: PlayKind::Supporter,
        needs: &[Cond::KnockedOutLastTurn { who: Who::Me, tag: Some(tag::TEAM_ROCKET), by_attack_damage: false }],
        // Each player shuffles their hand into their deck; you draw 5 cards and your opponent draws 3.
        steps: &[
            Step::new(Op::HandShuffleDraw(HandShuffleDrawSpec { who: Who::Opp, draw: Num::Lit(3) })),
            Step::new(Op::HandShuffleDraw(HandShuffleDrawSpec { who: Who::Me, draw: Num::Lit(5) })),
        ],
    }),
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
