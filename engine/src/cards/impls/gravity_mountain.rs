//! Gravity Mountain (SSP, stadium): each Stage 2 Pokémon in play (both
//! yours and your opponent's) gets -30 HP.
//!
//! Twinleaf: CheckHpEffect while this is the stadium in play, unless the
//! stadium effect is blocked for the target's owner; the stadium can't be used.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl {
    class: "GravityMountain",
    mask: mask(&[k::CHECK_HP, k::USE_STADIUM]),
    reduce,
    resume: None,
    coin: None,
    can_play: None,
};

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    match *g.e(e) {
        Effect::CheckHp { target, card, .. } if g.st.stadium_card() == Some(me) => {
            let owner = target.p as usize;
            if is_stadium_effect_blocked(g, owner, target, me) {
                return Ok(());
            }
            let stage2 = g.st.slot_pokemon(owner, target.s).map(|c| g.st.cdef(c).stage == Stage::Stage2 as u8).unwrap_or(false);
            // `effect.hp -= 30`: the setter writes hpBonus only when a Pokémon was captured.
            if stage2 && card.is_some() {
                g.st.players[owner].slots[target.s as usize].hp_bonus -= 30;
            }
            Ok(())
        }
        Effect::UseStadium { .. } if g.st.stadium_card() == Some(me) => bail!("CANNOT_USE_STADIUM"),
        _ => Ok(()),
    }
}
