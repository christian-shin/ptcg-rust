//! Antique Cover Fossil (SCR, support card): play this card as a 60 HP [C]
//! Basic Pokémon (at any time during your turn you may discard it from play);
//! Protective Cover — prevent all effects of attacks used by your opponent's
//! Pokémon done to this Pokémon. (Damage is not an effect.)
//!
//! Fixed (W1-D): the Trainer Ability used to be copied from the Poké Doll
//! (CANNOT_USE_POWER unless this card was the first card of the Active slot,
//! then put on the bottom of the deck via a toBottom MoveCardsEffect that also
//! duplicated it and dropped the deck's top card). It now discards this card
//! from play, with its attached cards, wherever it is (like Antique Root
//! Fossil), via one MoveCardsEffect of the whole slot. On its
//! own PlayItemEffect the card reduces a PlayPokemonEffect into the first
//! empty Bench slot; a RetreatEffect with it Active throws. Every attack
//! effect (AbstractAttackEffect) of an attack used by the opponent's Pokémon,
//! aimed at a slot holding this card as its top Pokémon, is prevented after a
//! lock probe for the owner, except Weakness/Resistance, Put Damage and Deal
//! Damage. Fixed (user 2026-10-08, as the text): the owner's own attacks are
//! no longer blocked. Fixed: it can't be affected by Special Conditions (printed text):
//! a `Prevent` over GainCondition (events batch 4: every cause, so the old sweep of conditions added
//! directly went), as for Antique Root Fossil.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "AntiqueCoverFossil",
    // Played from the hand as a 60 HP Basic Pokémon.
    play: Some(PlaySpec { kind: PlayKind::Item, needs: &[], steps: &[Step::new(Op::PlayAsPokemon(PlayAsPokemonSpec {}))] }),
    // At any time during your turn, you may discard it from play.
    powers: &[PowerSpec {
        index: 0,
        once: Once::No,
        needs: &[],
        steps: &[Step::new(Op::RemoveFromPlay(RemoveFromPlaySpec { slot: SlotExpr::This, destination: ZoneRef(Who::Me, Zone::Discard), effect_of_attack: false }))],
    }],
    passives: &[
        // Protective Cover: prevent all effects of attacks used by your opponent's Pokémon done to this Pokémon (every event
        // they cause, the switches included: APR C-04 / C-05, id2025, id2155).
        Passive { origin: RuleSource::Ability, modifier: Modifier::Prevent(PreventSpec::on(SlotPred::All(&[SlotPred::Holder, SlotPred::IsThisPokemon]), EFFECTS_OF_OPP_ATTACKS)) },
        // It can't be affected by Special Conditions and can't retreat.
        Passive {
            origin: RuleSource::CardRule,
            // Every GainCondition on it is prevented, whatever the cause (events batch 4).
            modifier: Modifier::Prevent(PreventSpec::on(SlotPred::All(&[SlotPred::Holder, SlotPred::IsThisPokemon]), EventPred::Kind(EventKind::GainCondition))),
        },
        Passive { origin: RuleSource::CardRule, modifier: Modifier::BlockUse(BlockUseSpec::RETREAT_THIS_ACTIVE) },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
