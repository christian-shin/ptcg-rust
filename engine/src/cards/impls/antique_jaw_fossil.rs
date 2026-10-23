//! Antique Jaw Fossil (POR 68): play this card as if it were a 60-HP Basic [C] Pokémon. It can't be affected by any Special Conditions and
//! can't retreat. At any time during your turn, you may discard it from play. Intimidating Jaw: as long as this Pokémon is in the Active Spot, attacks used by your opponent's Active Pokémon do 30 less damage (before applying Weakness and Resistance)
//!
//! Intimidating Jaw is Pyroar MEG 24's Intimidating Fang: a `DamageDealt` of -30 at the Deal stage on every attack of the opponent's Active Pokémon, to each Pokémon the attack damages (id158, id2213), while this Pokémon is in the Active Spot.
//!
//! The fossil rules are those of Antique Root Fossil: the play-as-Pokémon program, the discard action (power 0, not an Ability), a
//! `Prevent` over GainCondition whatever the cause, and the retreat lock. Intimidating Jaw is the card's printed Ability (power 1), so
//! the Ability readers see it.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "AntiqueJawFossil",
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
        // Intimidating Jaw: as long as it is in the Active Spot, attacks used by the opponent's Active Pokémon do 30 less damage
        // (the Deal stage, before Weakness and Resistance; to any Pokémon the attack damages, as Pyroar MEG 24: id158, id2213).
        Passive {
            origin: RuleSource::Ability,
            modifier: Modifier::DamageDealt(DamageDealtSpec {
                amount: -30,
                attacker: SlotPred::IsActive,
                side: Side::Opponent,
                opp_active_only: false,
                needs_damage: true,
                guard: Cond::IsActive(SlotExpr::This),
                ..DamageDealtSpec::DEFAULT
            }),
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
