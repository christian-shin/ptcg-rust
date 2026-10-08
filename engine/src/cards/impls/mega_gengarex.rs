//! Mega Gengar ex (MBG / PFL): Shadowy Concealment — if 1 of your [D]
//! Pokémon is Knocked Out by damage from an attack from your opponent's
//! Pokémon ex, that player takes 1 fewer Prize card (doesn't stack).
//! Void Gale — 230; move an Energy from this Pokémon to 1 of your Benched
//! Pokémon.
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
        // it doesn't stack. Today's behavior kept: the lock probe is made for the opponent.
        modifier: Modifier::PrizeAdjust(PrizeAdjustSpec {
            delta: -1,
            subject: SlotPred::TypeIs(crate::types::ct::DARK),
            by_attack_damage: true,
            attacker: SlotPred::Tag(crate::types::tag::POKEMON_EX_LOWER),
            nonstacking: Some("MEGA_GENGAR_SHADOW_HIDING_APPLIED"),
            owner_only: true,
            probe_opponent: true,
            ..PrizeAdjustSpec::DEFAULT
        }),
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
