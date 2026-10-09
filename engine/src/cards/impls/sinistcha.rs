//! Sinistcha (PBL / M5): Hide 'n' Sneak. Matcha Spin — if you have 6 or
//! more Pokémon with Hide 'n' Sneak in your discard pile, place 4 damage
//! counters on each of your opponent's Pokémon.
//!
//! Twinleaf: attack damage is zeroed first; then
//! PUT_X_DAMAGE_COUNTERS_ON_ALL_YOUR_OPPONENTS_POKEMON(4): one PutCountersEffect
//! (an effect of the attack) on the opponent's Active, then one per Benched
//! Pokémon. Fixed (phase 4b, R3): this used to be one PlaceDamageCountersEffect
//! (source = this card) per Pokémon, which Mist Energy and Spherical Shield
//! don't see.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Sinistcha",
    passives: &[
        Passive { origin: RuleSource::Ability, modifier: Modifier::PreventAttackEffects(HIDE_N_SNEAK) },
        // The opponent's attacks and Abilities switching this Pokémon in or out (ChangeActive; JP Q&A, Hariyama MEG 73).
        Passive { origin: RuleSource::Ability, modifier: Modifier::Prevent(HIDE_N_SNEAK_SWITCH) },
    ],
    attacks: &[AttackSpec {
        index: 0,
        steps: &[
            Step::before_damage(damage_is(Num::Lit(0))),
            Step::after_damage(Op::If(IfSpec {
                cond: Cond::Cmp(Num::CardCount(ZoneRef(Who::Me, Zone::Discard), Pred::HasAbilityNamed("Hide 'n' Sneak")), CmpOp::Ge, Num::Lit(6)),
                // 4 damage counters on each of the opponent's Pokémon (an effect of the attack).
                yes: &[Step::new(Op::EachSlot(EachSlotSpec { among: SlotSel::Pokemon(Who::Opp), what: EachWhat::Counters(CounterCause::Attack), amount: Num::Lit(4), ..EachSlotSpec::DEFAULT }))],
                no: &[],
            })),
        ],
    }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
