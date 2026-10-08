//! Durant ex (SSP 4): Sudden Shearing — when you play this Pokémon from your
//! hand onto your Bench, you may discard the top card of your opponent's
//! deck. Vengeful Crush — 120+; 30 more for each Prize card your opponent
//! has taken.
//!
//! Twinleaf: any PlayPokemonEffect for this card asks (ConfirmPrompt) unless
//! the Ability is blocked or the opposing deck is empty (aud-e); Vengeful Crush
//! sets `effect.damage = attack.damage + opponent.prizesTaken * 30`.
use crate::spec::prelude::*;

pub static SPEC: CardSpec = CardSpec {
    class: "Durantex",
    // Sudden Shearing: when you play this Pokémon from your hand onto your Bench, you may discard
    // the top card of your opponent's deck.
    triggers: &[Trigger {
        origin: RuleSource::Ability,
        event: Event::OnEnterPlay(OnEnterPlaySpec { method: EnterMethod::Play }),
        steps: &[Step::new(Op::May(MaySpec {
            asker: Who::Me,
            when: Cond::Nonempty(ZoneRef(Who::Opp, Zone::Deck), Pred::Any),
            msg: "WANT_TO_USE_ABILITY",
            yes: &[Step::new(Op::Move(MoveSpec {
                from: ZoneRef(Who::Opp, Zone::Deck),
                to: ZoneRef(Who::Opp, Zone::Discard),
                cards: CardSel::Top(Num::Lit(1)),
                ..MoveSpec::DEFAULT
            }))],
            no: &[],
        }))],
    }],
    // Vengeful Crush: 30 more damage for each Prize card your opponent has taken.
    attacks: &[AttackSpec { index: 0, steps: &[Step::before_damage(Op::Damage(DamageSpec { op: DamageOp::Add, hp: Num::Mul(&Num::PrizesTaken(Who::Opp), &Num::Lit(30)), when: Cond::True }))] }],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
