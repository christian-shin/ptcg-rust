//! Hop's Dubwool (JTG): Defiant Horn — when you play this Pokémon from your
//! hand to evolve 1 of your Pokémon during your turn, you may switch in 1
//! of your opponent's Benched Pokémon to the Active Spot. Headbutt — 80.
//!
//! Twinleaf: the ability-lock probe and the opposing-Bench check run before
//! the ConfirmPrompt (phase 4b fix: accepting with an empty opposing Bench
//! used to throw CANNOT_PLAY_THIS_CARD in the callback and end the game).
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "HopsDubwool",
    triggers: &[
        Trigger { origin: RuleSource::Ability, event: Event::OnEnterPlay(OnEnterPlaySpec { method: EnterMethod::Play }), steps: &[Step::new(Op::If(IfSpec { cond: Cond::All(&[Cond::Not(&Cond::AbilityBlocked), Cond::AnySlot(SlotSel::Bench(Who::Opp), SlotPred::Any)]), yes: &[Step::new(Op::May(MaySpec { asker: Who::Me, when: Cond::True, msg: "WANT_TO_USE_ABILITY", yes: &[Step::new(Op::Switch(SwitchSpec { side: Who::Opp, chooser: Who::Me, kind: SwitchKind::Silent, msg: "CHOOSE_POKEMON_TO_SWITCH", required: false }))], no: &[] }))], no: &[] }))] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
