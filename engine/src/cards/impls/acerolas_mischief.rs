//! Acerola's Mischief (MEG): usable only if your opponent has 2 or fewer
//! Prize cards left. Choose 1 of your Pokémon; during your opponent's next
//! turn, prevent all damage from and effects of attacks done to it by your
//! opponent's Pokémon ex.
//!
//! One `Prevent` (`DAMAGE_OR_EFFECTS`) on the chosen Pokémon, with the attacker's card as a Pokémon ex of the
//! opponent's as its cause: the attack's damage is zeroed at step 6 (APR C-16) and every event with an effect is
//! refused (Special Conditions, counters, switches, devolving, lasting effects: APR C-04 / C-05 / C-17, id2025, id2155).
//! The effect is declared by the card and put on the Pokémon (`Op::Declare`, a Trainer effect on a Pokémon: id2228): it stays when
//! the Pokémon moves to the Bench or switches (the JP FAQ's Ogerpon swap keeps it too), and ends with the opponent's next turn
//! (`DeclareUntil::OpponentsNextTurn`, in the EndTurn event before the end-of-turn effects, APR E-21). It also ends when the chosen Pokémon evolves or is
//! devolved (JP FAQ Acerola's Mischief ASC 180; scenarios acerolas-mischief-asc-180-ends-when-the-pokemon-{evolves,devolves}).
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "AcerolasMischief",
    play: Some(PlaySpec {
        kind: PlayKind::Supporter,
        needs: &[Cond::Printed(&Cond::Cmp(Num::PrizesLeft(Who::Opp), CmpOp::Le, Num::Lit(2)))],
        steps: &[
            Step::new(Op::PickSlot(PickSlotSpec { chooser: Who::Me, among: SlotSel::Pokemon(Who::Me), msg: "CHOOSE_POKEMON" })),
            // A Trainer effect on the Pokémon: it stays when the Pokémon moves to the Bench or switches.
            Step::new(Op::Declare(DeclareSpec { effect: 0, on: DeclareOn::Pokemon(SlotExpr::Picked), until: DeclareUntil::OpponentsNextTurn })),
        ],
    }),
    effects: &[
        // Prevent all damage from and effects of attacks from your opponent's Pokémon ex done to the chosen Pokémon (every
        // event they cause, the switches included: APR C-04 / C-05, id2025, id2155).
        Passive {
            origin: RuleSource::TrainerEffect,
            modifier: Modifier::Prevent(PreventSpec::on(
                SlotPred::Any,
                EventPred::All(&[DAMAGE_OR_EFFECTS, EventPred::Cause(CausePred::All(&[CausePred::By(Who::Opp), CausePred::Kind(crate::cause::CauseKind::Attack), CausePred::Card(Pred::PokemonEx)]))]),
            )),
        },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
