//! Crustle (DRI 12): Mysterious Rock Inn — prevent all damage done to this
//! Pokémon by attacks from your opponent's Pokémon ex. Superb Scissors — 120;
//! this attack's damage isn't affected by any effects on your opponent's Active
//! Pokémon.
//!
//! Rule: Mysterious Rock Inn is a `Prevent` over `Kind(Damage)` with the
//! attacker as the cause (`CausePred::Card(Tag(ex))`, read at step 6, APR C-16);
//! it prevents damage only, the attack's effects still happen (id2335: it is a
//! shield on Crustle, not an effect done to the attacker). Superb Scissors sets the
//! attack flag IgnoreDefenderEffects (Shred: it ignores a Prevent on the Defending
//! Pokémon, APR C-16).
use crate::spec::prelude::*;
use crate::types::tag;

pub static SPEC: CardSpec = CardSpec {
    class: "Crustle@DRI",
    attacks: &[AttackSpec { index: 0, steps: &[Step::before_damage(Op::AttackFlag(AttackFlagSpec { flag: AttackFlagKind::IgnoreDefenderEffects, value: true }))] }],
    passives: &[Passive {
        origin: RuleSource::Ability,
        modifier: Modifier::Prevent(PreventSpec::on(SlotPred::All(&[SlotPred::Holder, SlotPred::IsThisPokemon]), EventPred::All(&[DAMAGE_BY_OPP_ATTACKS, EventPred::Cause(CausePred::Card(Pred::Tag(tag::POKEMON_EX_LOWER)))]))),
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
