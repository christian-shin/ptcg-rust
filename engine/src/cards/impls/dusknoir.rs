//! Dusknoir (SFA): Cursed Blast — put 13 damage counters on 1 of your opponent's Pokémon, then this Pokémon is
//! Knocked Out. Shadow Bind — 150, the Defending Pokémon can't retreat during your opponent's next turn.
//!
//! Cursed Blast (an Ability, any number of times, no marker): a PlaceCounters event caused by the Ability, then
//! `Op::KnockOut` on this Pokémon. A refused placement (Hide 'n' Sneak, Battle Cage) places nothing and the Ability
//! still Knocks Dusknoir Out (id2264, id2425, JP FAQ Dusclops PRE 36 / Battle Cage). The Knock Out waits for the next
//! state check, after every effect (id2089: Togekiss's Wonder Kiss still flips; id2239: Prize and promotion order).
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
            Step::new(Op::KnockOut(KnockOutSpec { target: SlotExpr::This, when: Cond::True })),
        ],
    }],
    // Shadow Bind: the Defending Pokémon can't retreat during your opponent's next turn.
    attacks: &[AttackSpec { index: 0, steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::Lock(LastingLockSpec::on_defending(&CANT_RETREAT)) }))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
