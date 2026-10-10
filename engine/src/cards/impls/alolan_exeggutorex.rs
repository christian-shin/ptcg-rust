//! Alolan Exeggutor ex (SSP, Tera): Tropical Fever — 150; attach any number
//! of Basic Energy from your hand to your Pokémon in any way. Swinging
//! Sphene — flip a coin; heads: Knock Out the opponent's Active if it is
//! Basic; tails: Knock Out 1 of the opponent's Benched Pokémon.
//!
//! Twinleaf: tails opens a ChoosePokemonPrompt over the opponent's Bench with
//! the non-Basic Benched Pokémon blocked, and does nothing when no Benched
//! Basic exists (phase 4b fix: `blocked.push()` pushed nothing, so any Benched
//! Pokémon could be Knocked Out, and an empty Bench left the prompt
//! unanswerable). Tropical Fever's AttachEnergyPrompt has the default `max` =
//! hand size.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "AlolanExeggutorex",
    attacks: &[
        // Tropical Fever: attach any number of Basic Energy from your hand to your Pokémon in any way.
        AttackSpec {
            index: 0,
            steps: &[Step::after_damage(Op::If(IfSpec {
                cond: Cond::Nonempty(ZoneRef(Who::Me, Zone::Hand), Pred::BasicEnergy),
                yes: &[Step::new(Op::Attach(AttachSpec {
                    from: ZoneRef(Who::Me, Zone::Hand),
                    slots: AttachSlots::BenchActive,
                    bounds: Bounds { min: Num::Lit(0), max: Num::ZoneSize(ZoneRef(Who::Me, Zone::Hand)) },
                    route: AttachRoute::Effect,
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
                    mode: KnockOutMode::Opponent,
                    when: Cond::Slot(OPP_ACTIVE, SlotPred::Basic),
                }))],
                tails: &[
                    Step::new(Op::PickSlot(PickSlotSpec {
                        chooser: Who::Me,
                        among: SlotSel::Filtered(&SlotSel::Bench(Who::Opp), SlotPred::Basic),
                        msg: "CHOOSE_POKEMON_TO_DAMAGE",
                    })),
                    Step::new(Op::KnockOut(KnockOutSpec { target: SlotExpr::Picked, mode: KnockOutMode::Opponent, when: Cond::True })),
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
