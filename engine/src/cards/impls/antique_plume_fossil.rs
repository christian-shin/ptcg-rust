//! Antique Plume Fossil (WHT 79): play this card as if it were a 60-HP Basic [C] Pokémon. It can't be affected by any Special Conditions and
//! can't retreat. At any time during your turn, you may discard it from play. Plume Protection: as long as this Pokémon is on your Bench, prevent all damage done to this Pokémon by attacks from your opponent's Pokémon
//!
//! Plume Protection is a `Prevent` over the Damage event of the opponent's attacks on this Pokémon while it is Benched (the Tera rule's shape limited to the opponent's attacks, and an Ability, so an Ability lock turns it off); effects of those attacks still happen (APR C-17).
//!
//! The fossil rules are those of Antique Root Fossil: the play-as-Pokémon program, the discard action (power 0, not an Ability), a
//! `Prevent` over GainCondition whatever the cause, and the retreat lock. Plume Protection is the card's printed Ability (power 1), so
//! the Ability readers see it.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "AntiquePlumeFossil",
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
        // Plume Protection: while it is on the Bench, prevent all damage done to it by attacks from the opponent's Pokémon.
        Passive {
            origin: RuleSource::Ability,
            modifier: Modifier::Prevent(PreventSpec::on(SlotPred::All(&[SlotPred::Holder, SlotPred::IsThisPokemon, SlotPred::IsBench]), DAMAGE_BY_OPP_ATTACKS)),
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
