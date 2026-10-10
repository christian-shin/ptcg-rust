//! Antique Cover Fossil (SCR): play this card as if it were a 60-HP Basic [C] Pokémon; it can't be affected by any
//! Special Conditions and can't retreat; at any time during your turn, you may discard it from play. Protective Cover —
//! prevent all effects of attacks used by your opponent's Pokémon done to this Pokémon. (Damage is not an effect.)
//!
//! Protective Cover is one `Prevent` naming no kind (`EFFECTS_OF_OPP_ATTACKS`): every event with an effect the
//! opponent's attacks cause to it (Special Conditions, counters, discards, switches, a lasting effect, a Knock Out by an
//! effect), never Damage (APR C-17). "Can't be affected by any Special Conditions" is a `Prevent` over GainCondition,
//! whatever the cause (events batch 4). The discard action removes it from play with its attached cards (a LeavePlay
//! by its own effect). It can't retreat (`BlockUse::RETREAT_THIS_ACTIVE`).
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
        steps: &[Step::new(Op::RemoveFromPlay(RemoveFromPlaySpec { slot: SlotExpr::This, destination: ZoneRef(Who::Me, Zone::Discard) }))],
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
