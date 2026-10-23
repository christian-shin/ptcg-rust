//! Antique Sail Fossil (POR 69): play this card as if it were a 60-HP Basic [C] Pokémon. It can't be affected by any Special Conditions and
//! can't retreat. At any time during your turn, you may discard it from play. Protective Sail: whenever your opponent plays a Supporter card from their hand, prevent all effects of that card done to this Pokémon
//!
//! Protective Sail is a `Prevent` naming no kind (every event with an effect, never Damage: APR C-17) whose cause is a Supporter card of the opponent's Trainer effect (Mist Energy's shape with another cause).
//!
//! The fossil rules are those of Antique Root Fossil: the play-as-Pokémon program, the discard action (power 0, not an Ability), a
//! `Prevent` over GainCondition whatever the cause, and the retreat lock. Protective Sail is the card's printed Ability (power 1), so
//! the Ability readers see it.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "AntiqueSailFossil",
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
        // Protective Sail: the effects of a Supporter card the opponent plays from their hand done to this Pokémon don't happen (the
        // cause is that Trainer card, a Supporter, of the opponent's: no kind named, so every event with an effect, never Damage).
        Passive {
            origin: RuleSource::Ability,
            modifier: Modifier::Prevent(PreventSpec::on(
                SlotPred::All(&[SlotPred::Holder, SlotPred::IsThisPokemon]),
                EventPred::Cause(CausePred::All(&[CausePred::By(Who::Opp), CausePred::Kind(crate::cause::CauseKind::Trainer), CausePred::Card(Pred::Supporter)])),
            )),
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
