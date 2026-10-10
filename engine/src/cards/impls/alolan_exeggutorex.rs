//! Alolan Exeggutor ex (SSP 133, Tera): Tropical Frenzy — 150; you may attach any number of Basic Energy cards from
//! your hand to your Pokémon in any way you like. Swinging Sphene — flip a coin; heads: Knock Out your opponent's
//! Active Basic Pokémon; tails: Knock Out 1 of your opponent's Benched Basic Pokémon.
//!
//! Swinging Sphene's Knock Outs are by the attack's effect (`Op::KnockOut`): the KnockOut event's preventions are asked
//! (Mist Energy and the other "prevent all effects of attacks" stop it, id2427) and the Pokémon is Knocked Out at the
//! next state check with every other Knock Out, its Prize cards taken there (user decision D1; id2089, id810). Tails
//! asks for a Benched Basic Pokémon only, nothing without one. Tera: `TERA_RULE`.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "AlolanExeggutorex",
    attacks: &[
        // Tropical Frenzy: attach any number of Basic Energy from your hand to your Pokémon in any way.
        AttackSpec {
            index: 0,
            steps: &[Step::after_damage(Op::If(IfSpec {
                cond: Cond::Nonempty(ZoneRef(Who::Me, Zone::Hand), Pred::BasicEnergy),
                yes: &[Step::new(Op::Attach(AttachSpec {
                    from: ZoneRef(Who::Me, Zone::Hand),
                    slots: AttachSlots::BenchActive,
                    bounds: Bounds { min: Num::Lit(0), max: Num::ZoneSize(ZoneRef(Who::Me, Zone::Hand)) },
                    ..AttachSpec::DEFAULT
                }))],
                no: &[],
            }))],
        },
        // Swinging Sphene: heads, Knock Out the opponent's Active if it is Basic; tails, Knock Out
        // 1 of the opponent's Benched Basic Pokémon.
        AttackSpec {
            index: 1,
            steps: &[Step::after_damage(Op::Coin(CoinSpec {
                heads: &[Step::new(Op::KnockOut(KnockOutSpec {
                    target: OPP_ACTIVE,
                    when: Cond::Slot(OPP_ACTIVE, SlotPred::Basic),
                }))],
                tails: &[
                    Step::new(Op::PickSlot(PickSlotSpec {
                        chooser: Who::Me,
                        among: SlotSel::Filtered(&SlotSel::Bench(Who::Opp), SlotPred::Basic),
                        msg: "CHOOSE_POKEMON_TO_DAMAGE",
                    })),
                    Step::new(Op::KnockOut(KnockOutSpec { target: SlotExpr::Picked, when: Cond::True })),
                ],
                ..CoinSpec::DEFAULT
            }))],
        },
    ],
    // Tera: no attack damage while Benched.
    passives: &[Passive { origin: RuleSource::CardRule, modifier: Modifier::Prevent(TERA_RULE) }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
