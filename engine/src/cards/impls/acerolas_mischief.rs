//! Acerola's Mischief (MEG): usable only if your opponent has 2 or fewer
//! Prize cards left. Choose 1 of your Pokémon; during your opponent's next
//! turn, prevent all damage from and effects of attacks done to it by your
//! opponent's Pokémon ex.
//!
//! Twinleaf: every AbstractAttackEffect whose target carries this card's
//! marker is prevented when its player does not own the target and the
//! source's top Pokémon is an ex. The markers are cleared at the end of the
//! turn of the player holding the clear marker (the opponent).
use crate::cards::prelude::*;
use crate::marker;

pub static IMPL: CardImpl = CardImpl {
    class: "AcerolasMischief",
    mask: mask(&[
        k::TRAINER,
        k::END_TURN,
        k::APPLY_WEAKNESS,
        k::DEAL_DAMAGE,
        k::PUT_DAMAGE,
        k::AFTER_DAMAGE,
        k::PUT_COUNTERS,
        k::KNOCK_OUT_OPPONENT,
        k::DISCARD_CARDS,
        k::CARDS_TO_HAND,
        k::GUST_OPPONENT_BENCH,
        k::ADD_MARKER,
        k::ADD_SPECIAL_CONDITIONS,
        k::REMOVE_SPECIAL_CONDITIONS,
        k::HEAL_TARGET,
    ]),
    reduce,
    resume: Some(resume),
    coin: None,
    can_play: None,
};

fn mischief() -> crate::markers::MarkerName {
    marker!("ACEROLAS_MISCHIEF_MARKER")
}

fn clear_mischief() -> crate::markers::MarkerName {
    marker!("ACEROLAS_MISCHIEF_CLEAR_MARKER")
}

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if let Some(p) = trainer_played(g, e, me) {
        if g.st.players[p].supporter_turn > 0 {
            bail!("SUPPORTER_ALREADY_PLAYED");
        }
        if g.st.players[1 - p].prize_left() > 2 {
            bail!("CANNOT_PLAY_THIS_CARD");
        }
        let mut slots = SVec::new();
        slots.push(SlotType::Active as u8);
        slots.push(SlotType::Bench as u8);
        let mut f = CardFrame::at(1);
        f.a[0] = p as i32;
        let id = g.player_id(p);
        g.prompt(
            id,
            "CHOOSE_POKEMON",
            PromptKind::ChoosePokemon { player_type: PlayerType::BottomPlayer, slots, min: 1, max: 1, allow_cancel: false, blocked: TargetList::new() },
            Cont::Card { card: me, frame: f },
        );
        return Ok(());
    }

    if let Some(b) = g.e(e).atk_base().copied() {
        let t = b.target;
        if g.st.slot(t.p as usize, t.s).marker.has_from(mischief(), me) {
            let ex = g.st.slot_pokemon(b.source.p as usize, b.source.s).map(|c| g.st.cdef(c).has_tag(tag::POKEMON_EX_LOWER)).unwrap_or(false);
            if b.player != t.p && ex {
                g.set_prevent(e, true);
                return Ok(());
            }
        }
    }

    if let Effect::EndTurn { p } = *g.e(e) {
        let pu = p as usize;
        if g.st.players[pu].marker.has_from(clear_mischief(), me) {
            g.st.players[pu].marker.remove_from(clear_mischief(), me);
            let o = 1 - pu;
            for (s, _, _) in for_each_pokemon(g, o, PlayerType::BottomPlayer).iter() {
                g.st.players[o].slots[*s as usize].marker.remove_from(mischief(), me);
            }
        }
    }
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let p = f.a[0] as usize;
    let first = results.first().copied().unwrap_or(Res::Null);
    let sel = first.slots();
    let t = match sel.first() {
        Some(t) => *t,
        None => return Ok(()),
    };
    g.st.players[t.p as usize].slots[t.s as usize].marker.add(mischief(), me, crate::markers::SourceType::None, crate::markers::TargetScope::None);
    g.st.players[1 - p].marker.add(clear_mischief(), me, crate::markers::SourceType::None, crate::markers::TargetScope::None);
    Ok(())
}
