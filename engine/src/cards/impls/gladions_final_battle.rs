//! Gladion's Decisive Battle (M5, supporter): usable only as the last card
//! in your hand. During this turn, attacks used by your Pokémon without a
//! Rule Box do 80 more damage to the opponent's Active (before W/R).
//!
//! Twinleaf: the bonus applies to every DealDamageEffect of a marked player
//! targeting the opponent's Active whose source slot holds no Rule Box card;
//! each copy only honors its own marker.
use crate::cards::prelude::*;
use crate::marker;

pub static IMPL: CardImpl = CardImpl {
    class: "GladionsFinalBattle",
    mask: mask(&[k::TRAINER, k::DEAL_DAMAGE, k::END_TURN]),
    reduce,
    resume: None,
    coin: None,
    can_play: None,
};

fn gladion() -> crate::markers::MarkerName {
    marker!("M5_GLADIONS_DECISIVE_BATTLE")
}

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if let Some(p) = trainer_played(g, e, me) {
        if g.st.players[p].supporter_turn > 0 {
            bail!("SUPPORTER_ALREADY_PLAYED");
        }
        if g.st.players[p].hand.iter().any(|c| c != me) {
            bail!("CANNOT_PLAY_THIS_CARD");
        }
        g.st.players[p].marker.add(gladion(), me, crate::markers::SourceType::None, crate::markers::TargetScope::None);
    }

    if let Effect::DealDamage { b, .. } = *g.e(e) {
        let p = b.player as usize;
        let o = b.opponent as usize;
        if g.st.players[p].marker.has_from(gladion(), me) && b.target.p as usize == o && b.target.s == g.st.players[o].active {
            let rule_box = g.st.slot(b.source.p as usize, b.source.s).cards.iter().any(|c| g.st.cdef(c).has_rule_box());
            if !rule_box {
                if let Effect::DealDamage { damage, .. } = g.e_mut(e) {
                    *damage += 80;
                }
            }
        }
    }

    if let Effect::EndTurn { p } = *g.e(e) {
        let m = &mut g.st.players[p as usize].marker;
        if m.has_from(gladion(), me) {
            m.remove_from(gladion(), me);
        }
    }
    Ok(())
}
