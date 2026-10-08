//! Fezandipiti ex (SFA): Flip the Script — if any of your Pokémon were
//! Knocked Out during your opponent's last turn, draw 3 cards (one Flip the
//! Script per turn). Cruel Arrow — 100 damage to 1 of your opponent's Pokémon.
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "Fezandipitiex",
    attacks: &[
        AttackSpec { index: 0, steps: &[
            Step::after_damage(Op::DamageSlot(DamageSlotSpec { target: SlotTarget::Pick(PickSlotSpec { chooser: Who::Me, among: SlotSel::Pokemon(Who::Opp), msg: "CHOOSE_POKEMON_TO_DAMAGE" }), hp: Num::Lit(100), target_damage_mul: 0, calc: DamageCalc::Auto, when: Cond::True })),
        ] },
    ],
    powers: &[PowerSpec {
        index: 0,
        once: Once::PerTurnShared("FLIP_THE_SCRIPT_MARKER"),
        needs: &[Cond::KnockedOutLastTurn { who: Who::Me, by_attack_damage: false, tag: None }, Cond::Nonempty(ZoneRef(Who::Me, Zone::Deck), Pred::Any)],
        steps: &[
            Step::new(Op::Draw(DrawSpec { who: Who::Me, amount: DrawAmount::Count(Num::Lit(3)) })),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
