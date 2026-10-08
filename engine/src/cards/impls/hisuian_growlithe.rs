//! Hisuian Growlithe (TWM): Blazing Destruction — discard a Stadium in play.
//! Take Down — 40, this Pokémon also does 10 damage to itself.
//!
//! Twinleaf: with no Stadium in play the attack does nothing (phase 4b: it used
//! to throw CANNOT_USE_ATTACK, but an attack can be used with no effect);
//! the Stadium goes to its owner's discard (MOVE_CARDS of the whole list).
//! Take Down's recoil is a DealDamageEffect aimed at the attacker's Active.
use crate::spec::prelude::*;
pub static SPEC: CardSpec = CardSpec {
    class: "HisuianGrowlithe",
    attacks: &[
        AttackSpec { index: 0, steps: &[
            Step::after_damage(Op::DiscardStadium(DiscardStadiumSpec {})),
        ] },
        AttackSpec { index: 1, steps: &[
            Step::after_damage(self_damage(10)),
        ] },
    ],
    ..CardSpec::NONE
};

pub static IMPL: CardImpl = SPEC.card_impl();

// Still used by Mega Hawlucha ex until it is converted.
pub use legacy::{discard_stadium};

mod legacy {
    use crate::cards::prelude::*;

    /// `MOVE_CARDS(findCardList(stadium), findOwner(list).discard, { sourceCard })`.
    pub fn discard_stadium(g: &mut Game, stadium: CardId, me: CardId) -> R {
        let src = match g.st.locate(stadium) {
            Some(l) => l,
            None => bail!("INVALID_GAME_STATE"),
        };
        let owner = match src.owner() {
            Some(o) => o,
            None => bail!("INVALID_GAME_STATE"),
        };
        g.run_fx(Effect::MoveCards {
            source: src,
            destination: ListRef::Discard(owner as u8),
            cards: None,
            count: None,
            to_top: false,
            to_bottom: false,
            skip_cleanup: false,
            source_card: me,
        })?;
        Ok(())
    }
}
