//! Kieran (TWM, supporter): choose 1 — switch your Active with 1 of your
//! Benched Pokémon; or your attacks do 30 more damage to the opponent's
//! Active Pokémon ex / V this turn.
//!
//! Twinleaf quirks kept: the bonus checks only that the opponent's Active is
//! ex/V/VMAX/VSTAR, not the DealDamageEffect's target (bench hits of a
//! marked player also get +30). Without a Benched Pokémon the switch option
//! is removed from the SelectPrompt.
use crate::cards::prelude::*;
use crate::engine::turn::switch_pokemon_silent;
use crate::marker;

pub static IMPL: CardImpl = CardImpl {
    class: "Kieran",
    mask: mask(&[k::TRAINER, k::DEAL_DAMAGE, k::END_TURN]),
    reduce,
    resume: Some(resume),
    coin: None,
    can_play: None,
};

const BOTH: &[&str] = &["SWITCH_POKEMON", "INCREASE_DAMAGE_BY_30_AGAINST_OPPONENTS_EX_AND_V_POKEMON"];
const BOOST: &[&str] = &["INCREASE_DAMAGE_BY_30_AGAINST_OPPONENTS_EX_AND_V_POKEMON"];

fn kieran() -> crate::markers::MarkerName {
    marker!("KIERAN_MARKER")
}

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if let Effect::EndTurn { p } = *g.e(e) {
        let m = &mut g.st.players[p as usize].marker;
        if m.has_from(kieran(), me) {
            m.remove_from(kieran(), me);
            return Ok(());
        }
    }

    if let Effect::DealDamage { b, damage } = *g.e(e) {
        let p = b.player as usize;
        let o = 1 - p;
        if g.st.players[p].marker.has_from(kieran(), me) && damage > 0 {
            if let Some(oa) = g.st.active_pokemon(o) {
                let d = g.st.cdef(oa);
                if d.has_tag(tag::POKEMON_V) || d.has_tag(tag::POKEMON_VMAX) || d.has_tag(tag::POKEMON_VSTAR) || d.has_tag(tag::POKEMON_EX_LOWER) {
                    if let Effect::DealDamage { damage, .. } = g.e_mut(e) {
                        *damage += 30;
                    }
                }
            }
        }
    }

    if let Some(p) = trainer_played(g, e, me) {
        if g.st.players[p].supporter_turn > 0 {
            bail!("SUPPORTER_ALREADY_PLAYED");
        }
        move_cards(g, ListRef::Hand(p as u8), ListRef::Supporter(p as u8), &[me], me)?;
        g.set_prevent(e, true);
        let pl = &g.st.players[p];
        let has_bench = pl.bench.iter().any(|b| !pl.slots[*b as usize].cards.is_empty());
        let mut f = CardFrame::at(1);
        f.a[0] = p as i32;
        f.a[1] = has_bench as i32;
        let values = if has_bench { BOTH } else { BOOST };
        let id = g.player_id(p);
        g.prompt(
            id,
            "CHOOSE_OPTION",
            PromptKind::Select { values: SelectValues::Static(values), allow_cancel: false, default_value: 0 },
            Cont::Card { card: me, frame: f },
        );
    }
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    let p = f.a[0] as usize;
    let first = results.first().copied().unwrap_or(Res::Null);
    match f.stage {
        1 => {
            let choice = first.as_int();
            let has_bench = f.a[1] != 0;
            let n = if has_bench { 2 } else { 1 };
            if choice < 0 || choice >= n {
                // `options[choice]` is undefined: `option.action()` throws.
                bail!("TypeError: option is undefined");
            }
            if has_bench && choice == 0 {
                let mut slots = SVec::new();
                slots.push(SlotType::Bench as u8);
                let mut nf = CardFrame::at(2);
                nf.a[0] = p as i32;
                let id = g.player_id(p);
                g.prompt(
                    id,
                    "CHOOSE_POKEMON_TO_SWITCH",
                    PromptKind::ChoosePokemon { player_type: PlayerType::BottomPlayer, slots, min: 1, max: 1, allow_cancel: false, blocked: SVec::new() },
                    Cont::Card { card: me, frame: nf },
                );
            } else {
                g.st.players[p].marker.add(kieran(), me, crate::markers::SourceType::None, crate::markers::TargetScope::None);
            }
            Ok(())
        }
        2 => {
            let t = match first.slots().first() {
                Some(t) => *t,
                None => bail!("TypeError: result[0]"),
            };
            switch_pokemon_silent(g, p, t.s)?;
            Ok(())
        }
        _ => Ok(()),
    }
}
