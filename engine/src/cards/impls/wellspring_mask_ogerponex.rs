//! Wellspring Mask Ogerpon ex (TWM, Tera): Sob - 20; the Defending Pokémon
//! can't retreat during your opponent's next turn. Torrential Pump - 100;
//! you may shuffle 3 Energy from this Pokémon into your deck; if you do,
//! also 120 damage to 1 of your opponent's Benched Pokémon. Tera: no attack
//! damage while on the Bench.
//!
//! Twinleaf quirks kept: declining sets the attack's damage back to exactly
//! 100 (dropping any bonus added during the AttackEffect), and the energy
//! choice is priced as [C][C][C]. Twinleaf builds the energy map before the
//! confirm prompt; nothing can change the Active's energy in between, so it
//! is rebuilt when the confirm resolves (the frame can't hold the map).
//! R7A (ruling 1580): the Energy goes back into the deck, and the deck is shuffled, after the damage (`move_cards_after_damage`,
//! `shuffle_deck_after_damage`); the bench target is asked right after the Energy choice.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "WellspringMaskOgerponex",
    passives: &[Passive { origin: RuleSource::CardRule, modifier: Modifier::PreventDamage(PreventDamageSpec { how: PreventHow::Tera, ..PreventDamageSpec::DEFAULT }) }],
    attacks: &[
        AttackSpec { index: 0, steps: &[Step::after_damage(Op::Arm(ArmSpec { what: Lasting::PreventRetreat }))] },
        AttackSpec {
            index: 1,
            // You may shuffle 3 Energy from this Pokémon into your deck; if you do, also 120 damage to 1 of the opponent's Benched Pokémon.
            steps: &[Step::after_damage(Op::May(MaySpec {
                asker: Who::Me,
                when: Cond::AnySlot(SlotSel::Bench(Who::Opp), SlotPred::Any),
                msg: "WANT_TO_USE_ABILITY",
                yes: &[
                    Step::new(Op::DiscardEnergy(DiscardEnergySpec { selection: EnergySelection::Cost { n: Num::Lit(3), ty: ct::COLORLESS }, to: EnergyDest::Deck, ..DiscardEnergySpec::DEFAULT })),
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
