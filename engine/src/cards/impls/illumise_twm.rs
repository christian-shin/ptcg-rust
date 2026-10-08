//! Illumise (TWM): Slowing Perfume — only if you go second, during your
//! first turn: shuffle 1 of your opponent's Benched Pokémon and all attached
//! cards into their deck. Glide — 30.
//!
//! Twinleaf: legal only on `state.turn == 2` (throws CANNOT_USE_ATTACK
//! otherwise). Fixed (R1-9): it shuffled the opponent's *Active* Pokémon
//! (and ran `clearEffects()` on the vacated Active slot); it now does nothing
//! with an empty opposing Bench, otherwise the attacker chooses 1 Benched
//! Pokémon (ChoosePokemonPrompt, min 1, max 1, no cancel), MOVE_CARDS moves
//! the whole stack into the opponent's deck, then the opponent's deck
//! shuffle is prompted (like Sylveon ex's Angelite).
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "Illumise@TWM",
    attacks: &[
        AttackSpec { index: 0, steps: &[
            Step::before_damage(Op::Fail(FailSpec { unless: Cond::Cmp(Num::Turn, CmpOp::Eq, Num::Lit(2)), error: "CANNOT_USE_ATTACK" })),
            Step::after_damage(Op::If(IfSpec { cond: Cond::AnySlot(SlotSel::Bench(Who::Opp), SlotPred::Any), yes: &[Step::new(Op::RemovePicked(RemovePickedSpec { among: SlotSel::Bench(Who::Opp), msg: "CHOOSE_POKEMON_TO_SHUFFLE" })), Step::new(Op::Shuffle(ShuffleSpec { zone: ZoneRef(Who::Opp, Zone::Deck) }))], no: &[] })),
        ] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
