//! Flygon ex (SSP 106, Tera): Reversing Storm - 130; you may switch this Pokémon with 1 of your Benched Pokémon. Sonic Peridot - this
//! attack does 100 damage to each of your opponent's Pokémon ex and Pokémon V. This attack's damage isn't affected by Weakness or
//! Resistance.
//!
//! The Tera rule is `TERA_RULE` (no damage from attacks while this Pokémon is on the Bench). Reversing Storm: a yes / no after the
//! damage, then the switch (Pokémon Catcher-style `switch_self`; "if you do" isn't printed). Sonic Peridot: a Damage event of 100
//! on every Pokémon of the opponent's whose card is a Pokémon ex or a Pokémon V (the Active Pokémon too), with no Weakness or
//! Resistance (`DamageCalc::Put`: the printed text names both for the Active Pokémon as well); the attack has no damage of its own.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "FlygonEx",
    attacks: &[
        AttackSpec { index: 0, steps: &[Step::after_damage(Op::May(MaySpec { asker: Who::Me, when: Cond::True, yes: &[Step::new(switch_self())], no: &[] }))] },
        AttackSpec {
            index: 1,
            steps: &[Step::after_damage(Op::EachSlot(EachSlotSpec {
                among: SlotSel::Filtered(&SlotSel::Pokemon(Who::Opp), SlotPred::Top(Pred::OneOf(&[Pred::PokemonEx, Pred::PokemonV]))),
                what: EachWhat::Damage(DamageCalc::Put),
                amount: Num::Lit(100),
                ..EachSlotSpec::DEFAULT
            }))],
        },
    ],
    passives: &[Passive { origin: RuleSource::CardRule, modifier: Modifier::Prevent(TERA_RULE) }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
