//! Dusknoir (SFA): Cursed Blast — put 13 damage counters on 1 of your
//! opponent's Pokémon, then this Pokémon is Knocked Out. Shadow Bind — 150,
//! the Defending Pokémon can't retreat during your opponent's next turn.
//!
//! Same Twinleaf structure as Dusclops (no once-per-turn marker, `damage += 999`).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Dusknoir",
    // Cursed Blast: put 13 damage counters on 1 of your opponent's Pokémon, then this Pokémon is
    // Knocked Out.
    powers: &[PowerSpec {
        index: 0,
        once: Once::No,
        needs: &[],
        steps: &[
            Step::new(Op::PlaceCounters(PlaceCountersSpec {
                target: SlotTarget::Pick(PickSlotSpec { chooser: Who::Me, among: SlotSel::PokemonBenchFirst(Who::Opp), msg: "CHOOSE_POKEMON_TO_DAMAGE" }),
                counters: Num::Lit(13)
            })),
            Step::new(Op::KnockOut(KnockOutSpec { target: SlotExpr::This, mode: KnockOutMode::Direct, when: Cond::True })),
        ],
    }],
    // Shadow Bind: the Defending Pokémon can't retreat during your opponent's next turn.
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::PreventRetreat }))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
