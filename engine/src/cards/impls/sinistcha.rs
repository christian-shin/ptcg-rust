//! Sinistcha (PBL / M5): Hide 'n' Sneak. Matcha Spin — if you have 6 or
//! more Pokémon with Hide 'n' Sneak in your discard pile, place 4 damage
//! counters on each of your opponent's Pokémon.
//!
//! Hide 'n' Sneak is `Modifier::Prevent(HIDE_N_SNEAK)` (see shuppet.rs). Matcha Spin does no damage; the counters are one
//! PlaceCounters event per Pokémon with the attack as its cause (APR C-07: no Weakness, Resistance or damage modifiers).
//! Mist Energy or Hide 'n' Sneak on a Pokémon refuses its counters and the others still get theirs.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Sinistcha",
    passives: &[
        Passive { origin: RuleSource::Ability, modifier: Modifier::Prevent(HIDE_N_SNEAK) },
    ],
    attacks: &[AttackSpec {
        index: 0,
        steps: &[
            Step::before_damage(damage_is(Num::Lit(0))),
            Step::after_damage(Op::If(IfSpec {
                cond: Cond::Cmp(Num::CardCount(ZoneRef(Who::Me, Zone::Discard), Pred::HasAbilityNamed("Hide 'n' Sneak")), CmpOp::Ge, Num::Lit(6)),
                // 4 damage counters on each of the opponent's Pokémon (an effect of the attack).
                yes: &[Step::new(Op::EachSlot(EachSlotSpec { among: SlotSel::Pokemon(Who::Opp), what: EachWhat::Counters, amount: Num::Lit(4), ..EachSlotSpec::DEFAULT }))],
                no: &[],
            })),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
