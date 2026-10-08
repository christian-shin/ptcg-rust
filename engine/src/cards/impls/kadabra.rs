//! Kadabra (M1S): Psychic Draw — when you play this Pokémon from your hand to
//! evolve, you may draw 2 cards. Super Psy Bolt — 30.
//!
//! Twinleaf: fires on any EvolveEffect for this card (Rare Candy included).
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "Kadabra",
    triggers: &[
        Trigger { origin: RuleSource::Ability, event: Event::OnEnterPlay(OnEnterPlaySpec { method: EnterMethod::Evolve }), steps: &[Step::new(Op::If(IfSpec { cond: Cond::All(&[Cond::Nonempty(ZoneRef(Who::Me, Zone::Deck), Pred::Any), Cond::Not(&Cond::AbilityBlocked)]), yes: &[Step::new(Op::May(MaySpec { asker: Who::Me, when: Cond::True, msg: "WANT_TO_USE_ABILITY", yes: &[Step::new(Op::Draw(DrawSpec { who: Who::Me, amount: DrawAmount::Count(Num::Lit(2)) }))], no: &[] }))], no: &[] }))] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();
