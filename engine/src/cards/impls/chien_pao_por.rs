//! Chien-Pao (POR 54): Strafe — 20; you may switch this Pokémon with 1 of your Benched Pokémon. Rising Blade — 80+; if your
//! opponent's Active Pokémon is a Pokémon ex, this attack does 80 more damage.
//!
//! Strafe's choice is a `May` asked only with a Benched Pokémon (a yes that can't change the game isn't asked, APR B-01, id925); yes
//! is a ChangeActive (Switch, your own, APR C-03), as Mega Latias ex MEG 100. Rising Blade adds 80 when the Defending Pokémon is a
//! Pokémon ex.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "ChienPaoPOR",
    attacks: &[
        AttackSpec {
            index: 0,
            steps: &[Step::after_damage(Op::May(MaySpec {
                asker: Who::Me,
                when: Cond::AnySlot(SlotSel::Bench(Who::Me), SlotPred::Any),
                yes: &[Step::new(switch_self())],
                no: &[],
            }))],
        },
        AttackSpec { index: 1, steps: &[Step::before_damage(more_damage_if(80, Cond::Slot(OPP_ACTIVE, SlotPred::Top(Pred::PokemonEx))))] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
