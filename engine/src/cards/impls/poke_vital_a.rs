//! Poké Vital A (SFA, ACE SPEC): heal 150 damage from 1 of your Pokémon. This card can't be put into your hand or deck
//! from the discard pile.
//!
//! "Can't be put" is a lock over this card's PutIntoHand / PutIntoDeck from the discard pile (events batch 7), whoever's
//! effect puts it: the card stays and the other cards move.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "PokeVitalA",
    play: Some(PlaySpec {
        kind: PlayKind::Item,
        needs: &[],
        steps: &[Step::new(Op::Heal(HealSpec {
            target: SlotTarget::Pick(PickSlotSpec { chooser: Who::Me, among: SlotSel::Filtered(&SlotSel::Pokemon(Who::Me), SlotPred::Damaged), msg: "CHOOSE_POKEMON_TO_HEAL" }),
            hp: Num::Lit(150),
            clear_conditions: false,
        }))],
    }),
    passives: &[Passive { origin: RuleSource::CardRule, modifier: Modifier::BlockUse(BlockUseSpec::NOT_FROM_DISCARD_TO_HAND_OR_DECK) }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
