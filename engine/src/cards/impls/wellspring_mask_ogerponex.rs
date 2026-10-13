//! Wellspring Mask Ogerpon ex (TWM, Tera): Sob - 20; the Defending Pokémon
//! can't retreat during your opponent's next turn. Torrential Pump - 100;
//! you may shuffle 3 Energy from this Pokémon into your deck; if you do,
//! also 120 damage to 1 of your opponent's Benched Pokémon. Tera: no attack
//! damage while on the Bench.
//!
//! The Tera rule is `TERA_RULE`: a `Prevent` over `Kind(Damage)` on this Pokémon while it is on the Bench. Torrential Pump
//! asks the Energy first (priced as [C][C][C]) and the bench target right after, the main damage is done, then the Energy
//! goes back into the deck and the deck is shuffled (id2031); the 120 is one Damage event on the chosen Benched
//! Pokémon (no Weakness or Resistance).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "WellspringMaskOgerponex",
    passives: &[Passive { origin: RuleSource::CardRule, modifier: Modifier::Prevent(TERA_RULE) }],
    attacks: &[
        AttackSpec { index: 0, steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::Lock(LastingLockSpec::on_defending(&CANT_RETREAT)) }))] },
        AttackSpec {
            index: 1,
            // You may shuffle 3 Energy from this Pokémon into your deck; if you do, also 120 damage to 1 of the opponent's Benched Pokémon.
            steps: &[Step::after_damage(Op::May(MaySpec {
                asker: Who::Me,
                // 3 Energy must be attached to this Pokémon to shuffle (a copycat's own Energy, JP FAQ Clefable), and a
                // Benched Pokémon must be there to damage.
                when: Cond::All(&[
                    Cond::AnySlot(SlotSel::Bench(Who::Opp), SlotPred::Any),
                    Cond::Cmp(Num::EnergyOn(SlotSel::One(MY_ACTIVE), EnergyUnit::ProvidedUnits), CmpOp::Ge, Num::Lit(3)),
                ]),
                msg: "WANT_TO_USE_ABILITY",
                yes: &[
                    Step::new(Op::DiscardEnergy(DiscardEnergySpec { selection: EnergySelection::Cost { n: Num::Lit(3), ty: ct::COLORLESS }, to: EnergyDest::Deck, ..DiscardEnergySpec::DEFAULT })),
                    // "If you do": the 3 Energy are there (the `when` above), so they go into the deck (the removal waits for the
                    // damage and reports Done when queued, so nothing reads Done after it).
                    Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Me, Zone::Deck), wait: true })),
                    Step::new(Op::DamageSlot(DamageSlotSpec {
                        target: SlotTarget::Pick(PickSlotSpec { chooser: Who::Me, among: SlotSel::Bench(Who::Opp), msg: "CHOOSE_POKEMON_TO_DAMAGE" }),
                        hp: Num::Lit(120),
                        target_damage_mul: 0,
                        calc: DamageCalc::Put,
                        when: Cond::True,
                    })),
                ],
                no: &[],
            }))],
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
