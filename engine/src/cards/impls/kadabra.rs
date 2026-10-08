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

// Still used by Alakazam until it is converted.
pub use legacy::{psychic_draw_reduce,psychic_draw_resume};

mod legacy {
    use crate::cards::prelude::*;

    /// `JUST_EVOLVED(effect, this)` → CONFIRMATION_PROMPT unless the ability is blocked.
    pub fn psychic_draw_reduce(g: &mut Game, me: CardId, e: EffId) -> R {
        let p = match *g.e(e) {
            Effect::Evolve { p, card, .. } if card == me => p as usize,
            _ => return Ok(()),
        };
        // An Ability can't be used for no effect: the number of cards in a deck is public (Advanced Rulebook E-05,
        // rulings 244, 782).
        if g.st.players[p].deck.is_empty() {
            return Ok(());
        }
        if is_ability_blocked(g, p, me, None) {
            return Ok(());
        }
        let mut f = CardFrame::at(1);
        f.a[0] = p as i32;
        confirmation_prompt(g, p, "WANT_TO_USE_ABILITY", Cont::Card { card: me, frame: f });
        Ok(())
    }

    pub fn psychic_draw_resume(g: &mut Game, f: CardFrame, results: &[Res], n: usize) -> R {
        if f.stage != 1 {
            return Ok(());
        }
        if results.first().map(|r| r.as_bool()).unwrap_or(false) {
            draw_cards(g, f.a[0] as usize, n)?;
        }
        Ok(())
    }
}
