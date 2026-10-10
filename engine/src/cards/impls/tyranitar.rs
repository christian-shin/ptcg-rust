//! Tyranitar (JTG): Daunting Gaze — while this Pokémon is in the Active
//! Spot, your opponent can't play Item cards from their hand. Crackling
//! Stomp — 150; discard the top 2 cards of your opponent's deck.
//!
//! Events batch 7: a lock over the opponent's PlayTrainer of an Item from the hand (`PLAY_ITEM_FROM_HAND`), while this
//! Pokémon is Active and has its Ability.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Tyranitar",
    // Daunting Gaze: while this Pokémon is in the Active Spot, your opponent can't play Item cards from their hand.
    passives: &[Passive { origin: RuleSource::Ability, modifier: Modifier::BlockUse(BlockUseSpec {
        binds: Binds::Opponent,
        lock: LockDecl::on(PLAY_ITEM_FROM_HAND, "BLOCKED_BY_ABILITY"),
        while_: &[LockWhile::Active],
        ability: true,
    }) }],
    attacks: &[AttackSpec {
        index: 0,
        // Crackling Stomp: discard the top 2 cards of your opponent's deck.
        steps: &[Step::after_damage(Op::Discard(DiscardSpec { from: ZoneRef(Who::Opp, Zone::Deck), cards: CardSel::Top(Num::Lit(2)), ..DiscardSpec::DEFAULT }))],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
