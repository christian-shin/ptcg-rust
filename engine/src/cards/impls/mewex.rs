//! Mew ex (30C 66): Memory Helix - this Pokémon can use the attacks of any of your Benched Pokémon. (You still need the
//! necessary Energy to use each attack.) Teleportation Burst - 30; you may switch this Pokémon with 1 of your Benched Pokémon.
//!
//! Memory Helix is a `BenchAttacks` declaration read in the derived attack list while this Pokémon is Active and the Ability
//! works: each Benched Pokémon's attack is offered as a copied one, run by the copy session of `copy_attack.rs` (the copied
//! attack's own text, with the Energy cost paid by Mew ex). Another Pokémon's cost change binds its own holder (Seasoned Skill,
//! Plasma Bane: JP FAQ Bloodmoon Ursaluna ex; scenario mew-ex-30c-66-memory-helix-blood-moon-pays-its-full-cost).
//! Teleportation Burst is a `May` after the damage (a ChangeActive, Switch, APR C-03).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Mewex",
    attacks: &[AttackSpec {
        index: 0,
        steps: &[Step::after_damage(Op::May(MaySpec {
            asker: Who::Me,
            when: Cond::AnySlot(SlotSel::Bench(Who::Me), SlotPred::Any),
            msg: "WANT_TO_SWITCH_POKEMON",
            yes: &[Step::new(Op::Switch(SwitchSpec { change: ActiveChange::Switch, among: SwitchAmong::Bench, msg: "CHOOSE_NEW_ACTIVE_POKEMON", required: false }))],
            no: &[],
        }))],
    }],
    passives: &[Passive { origin: RuleSource::Ability, modifier: Modifier::BenchAttacks(BenchAttacksSpec {}) }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
