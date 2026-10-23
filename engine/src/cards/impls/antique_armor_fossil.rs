//! Antique Armor Fossil (PBL 72): play this card as if it were a 60-HP Basic [C] Pokémon. It can't be affected by any Special Conditions and
//! can't retreat. At any time during your turn, you may discard it from play. Protective Armor: as long as this Pokémon is in the Active Spot, all of your Pokémon take 10 less damage from attacks from your opponent's Pokémon (after applying Weakness and Resistance)
//!
//! Protective Armor is a `DamageTaken` (10) over every Pokémon of the owner, after Weakness and Resistance, only for damage from the opponent's Pokémon's attacks, while this Pokémon is in the Active Spot; it is an Ability, so an Ability lock turns it off.
//!
//! The fossil rules are those of Antique Root Fossil: the play-as-Pokémon program, the discard action (power 0, not an Ability), a
//! `Prevent` over GainCondition whatever the cause, and the retreat lock. Protective Armor is the card's printed Ability (power 1), so
//! the Ability readers see it.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "AntiqueArmorFossil",
    // Played from the hand as a 60 HP Basic Pokémon.
    play: Some(PlaySpec { kind: PlayKind::Item, needs: &[], steps: &[Step::new(Op::PlayAsPokemon(PlayAsPokemonSpec {}))] }),
    // At any time during your turn, you may discard it from play.
    powers: &[PowerSpec {
        index: 0,
        once: Once::No,
        needs: &[],
        steps: &[Step::new(Op::RemoveFromPlay(RemoveFromPlaySpec { slot: SlotExpr::This, destination: ZoneRef(Who::Me, Zone::Discard), top: false }))],
    }],
    passives: &[
        // Protective Armor: as long as it is in the Active Spot, each of your Pokémon takes 10 less damage from the opponent's attacks.
        Passive {
            origin: RuleSource::Ability,
            modifier: Modifier::DamageTaken(DamageTakenSpec { amount: 10, subject: SlotPred::Any, side: Side::Owner, guard: Cond::IsActive(SlotExpr::This), ..DamageTakenSpec::DEFAULT }),
        },
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
