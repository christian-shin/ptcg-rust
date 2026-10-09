//! Tyranitar (JTG): Daunting Gaze — while this Pokémon is in the Active
//! Spot, your opponent can't play Item cards from their hand. Crackling
//! Stomp — 150; discard the top 2 cards of your opponent's deck.
//!
//! Twinleaf: every PlayItemEffect is inspected while this card is in play
//! on either side. If it is the item player's own Active (and not the
//! opponent's) nothing happens; if it is the opponent's Active, a real
//! PowerEffect for the *item player* probes the ability lock and
//! BLOCKED_BY_ABILITY is thrown when it passes.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Tyranitar",
    // Daunting Gaze: while this Pokémon is in the Active Spot, your opponent can't play Item cards from their hand.
    passives: &[Passive { origin: RuleSource::Ability, modifier: Modifier::BlockUse(BlockUseSpec {
        binds: Binds::Opponent,
        lock: LockDecl { actions: &[LockedAction::PlayItem], card: Pred::Any, except: Pred::False, error: "BLOCKED_BY_ABILITY", ..LockDecl::NONE },
        while_: &[LockWhile::Active],
        ability: true,
    }) }],
    attacks: &[AttackSpec {
        index: 0,
        // Crackling Stomp: discard the top 2 cards of your opponent's deck.
        steps: &[Step::after_damage(Op::Move(MoveSpec { from: ZoneRef(Who::Opp, Zone::Deck), to: ZoneRef(Who::Opp, Zone::Discard), cards: CardSel::Top(Num::Lit(2)), ..MoveSpec::DEFAULT }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
