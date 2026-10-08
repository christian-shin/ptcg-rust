//! Mew ex (30C / M6a): Memory Helix — a passive Ability: this Pokémon can use
//! the attacks of any of your Benched Pokémon. They are added to its attack
//! options (CheckPokemonAttacksEffect.copiedAttacks) while it is Active and
//! the Ability isn't blocked; the AttackAction then runs the chosen one with
//! `delegateFrom` (see `copy_attack.rs`). Teleportation Burst — 30, you may
//! switch this Pokémon with 1 of your Benched Pokémon.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Mewex",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::after_damage(Op::May(MaySpec {
            asker: Who::Me,
            when: Cond::AnySlot(SlotSel::Bench(Who::Me), SlotPred::Any),
            msg: "WANT_TO_SWITCH_POKEMON",
            yes: &[Step::new(Op::Switch(SwitchSpec { side: Who::Me, chooser: Who::Me, kind: SwitchKind::Plain, msg: "CHOOSE_NEW_ACTIVE_POKEMON", required: false }))],
            no: &[],
        }))],
    }],
    passives: &[Passive { origin: RuleSource::Ability, modifier: Modifier::BenchAttacks(BenchAttacksSpec {}) }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
