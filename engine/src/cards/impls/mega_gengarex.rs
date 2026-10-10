//! Mega Gengar ex (MBG / PFL): Shadowy Concealment — if 1 of your [D]
//! Pokémon is Knocked Out by damage from an attack from your opponent's
//! Pokémon ex, that player takes 1 fewer Prize card (doesn't stack).
//! Void Gale — 230; move an Energy from this Pokémon to 1 of your Benched
//! Pokémon.
//!
//! Rule: Shadowy Concealment is a `PrizeAdjust` over the KnockOut view: it applies to
//! a Knock Out of your [D] Pokémon (`owner_only`) whose `ko_by` is AttackDamage and
//! whose attacker, where it is now, is a Pokémon ex (a Mega Evolution ex too, id2244);
//! it is read when the Prizes are taken, so it works for a Knock Out that Knocks this
//! Pokémon out too (id2262), and it doesn't stack (`nonstacking`: one reduction per
//! Knocked Out Pokémon, however many copies). Void Gale's Energy move is a MoveEnergy
//! event after the damage (cause: this attack, APR C-10 onto your own Pokémon).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "MegaGengarex",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::after_damage(Op::If(IfSpec {
            cond: Cond::AnySlot(SlotSel::Bench(Who::Me), SlotPred::Any),
            yes: &[
                Step::new(Op::PickSlot(PickSlotSpec { chooser: Who::Me, among: SlotSel::One(MY_ACTIVE), msg: "" })),
                Step::new(Op::Attach(AttachSpec {
                    from: ZoneRef(Who::Me, Zone::Attached(SlotExpr::Picked)),
                    predicate: Pred::Energy,
                    slots: AttachSlots::Bench,
                    bounds: Bounds { min: Num::Lit(1), max: Num::Lit(1) },
                    ..AttachSpec::DEFAULT
                })),
            ],
            no: &[],
        }))],
    }],
    passives: &[Passive {
        origin: RuleSource::Ability,
        // 1 fewer Prize card for a Knock Out of your [D] Pokémon by damage from an opponent's Pokémon ex;
        // it doesn't stack.
        modifier: Modifier::PrizeAdjust(PrizeAdjustSpec {
            delta: -1,
            subject: SlotPred::TypeIs(crate::types::ct::DARK),
            by_attack_damage: true,
            attacker: SlotPred::Tag(crate::types::tag::POKEMON_EX_LOWER),
            nonstacking: Some("MEGA_GENGAR_SHADOW_HIDING_APPLIED"),
            owner_only: true,
            ..PrizeAdjustSpec::DEFAULT
        }),
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
