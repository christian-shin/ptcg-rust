//! Antique Skull Fossil (PBL 73): play this card as if it were a 60-HP Basic [C] Pokémon. It can't be affected by any Special Conditions and
//! can't retreat. At any time during your turn, you may discard it from play. Spiny Skull: if this Pokémon is in the Active Spot and is damaged by an attack from your opponent's Pokémon (even if this Pokémon is Knocked Out), place 3 damage counters on the Attacking Pokémon
//!
//! Spiny Skull is Bruxish SSP 49's Counterattack: a trigger over the Damage event of the opponent's attacks on this Pokémon while it is the Active Pokémon (even if it is Knocked Out), placing 3 damage counters on the Attacking Pokémon.
//!
//! The fossil rules are those of Antique Root Fossil: the play-as-Pokémon program, the discard action (power 0, not an Ability), a
//! `Prevent` over GainCondition whatever the cause, and the retreat lock. Spiny Skull is the card's printed Ability (power 1), so
//! the Ability readers see it.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "AntiqueSkullFossil",
    // Played from the hand as a 60 HP Basic Pokémon.
    play: Some(PlaySpec { kind: PlayKind::Item, needs: &[], steps: &[Step::new(Op::PlayAsPokemon(PlayAsPokemonSpec {}))] }),
    // At any time during your turn, you may discard it from play.
    powers: &[PowerSpec {
        index: 0,
        once: Once::No,
        needs: &[],
        steps: &[Step::new(Op::RemoveFromPlay(RemoveFromPlaySpec { slot: SlotExpr::This, destination: ZoneRef(Who::Me, Zone::Discard), top: false }))],
    }],
    triggers: &[Trigger {
        origin: RuleSource::Ability,
        // Spiny Skull: a trigger over the Damage event of an opponent's attack on this Pokémon while it is the Active Pokémon
        // (`DamagedActive`, id1992): 3 damage counters by the Ability (a PlaceCounters event, not damage) on the Attacking Pokémon
        // wherever it is now, nothing when it left play (Bruxish SSP 49's Counterattack).
        event: Event::On(EventPred::All(&[DAMAGE_BY_OPP_ATTACKS, EventPred::Slot(SlotPred::Holder), EventPred::DamagedActive])),
        steps: &[Step::new(Op::PlaceCounters(PlaceCountersSpec { target: SlotTarget::Slot(SlotExpr::CausePokemon), counters: Num::Lit(3) }))],
    }],
    passives: &[
        // It can't be affected by Special Conditions and can't retreat.
        Passive {
            origin: RuleSource::CardRule,
            modifier: Modifier::Prevent(PreventSpec::on(SlotPred::All(&[SlotPred::Holder, SlotPred::IsThisPokemon]), EventPred::Kind(EventKind::GainCondition))),
        },
        Passive { origin: RuleSource::CardRule, modifier: Modifier::BlockUse(BlockUseSpec::RETREAT_THIS_ACTIVE) },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
