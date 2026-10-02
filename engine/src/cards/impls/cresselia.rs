//! Cresselia (SFA): Healing Pirouette — heal 20 damage from each of your
//! Pokémon. Crescent Purge — 80+; you may turn 1 of your face-down Prize
//! cards face up for 80 more damage.
//!
//! Twinleaf: the "face-down" test is `prizes.filter(p => p.isSecret)` over all
//! six prize lists (empty ones included), but the ChoosePrizePrompt (count 1,
//! cancellable) can pick any non-empty Prize; picking one that is already face
//! up throws CANNOT_USE_POWER, and cancelling throws (`chosenPrize[0]` on
//! null). The chosen list gets isSecret = false, faceUpPrize = true.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Cresselia", mask: mask(&[k::ATTACK]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        for (s, _, _) in for_each_pokemon(g, p, PlayerType::BottomPlayer).iter().copied() {
            g.run_fx(Effect::Heal { p: p as u8, target: SlotRef::new(p, s), damage: 20 })?;
        }
    }
    if was_attack_used(g, e, 1, me) {
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        let pl = &g.st.players[p];
        let secret = (0..pl.prize_count as usize).filter(|i| !pl.prize_public[*i]).count();
        if secret > 0 {
            g.retain_fx(e);
            let mut f = CardFrame::at(1);
            f.a[0] = p as i32;
            f.e[0] = e;
            confirmation_prompt(g, p, "WANT_TO_USE_ABILITY", Cont::Card { card: me, frame: f });
        }
    }
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    let p = f.a[0] as usize;
    let atk = f.e[0];
    let first = results.first().copied().unwrap_or(Res::Null);
    match f.stage {
        1 => {
            if !first.as_bool() {
                g.release_fx(atk);
                return Ok(());
            }
            let mut nf = CardFrame::at(2);
            nf.a[0] = p as i32;
            nf.e[0] = atk;
            let id = g.player_id(p);
            g.prompt(
                id,
                "CHOOSE_POKEMON",
                PromptKind::ChoosePrize { count: 1, blocked: SVec::new(), use_opponent_prizes: false, allow_cancel: true, is_secret: false, destination: None },
                Cont::Card { card: me, frame: nf },
            );
            Ok(())
        }
        2 => {
            let r = (|| -> R {
                let idx = match first {
                    Res::Prizes(v) => v.get(0).copied(),
                    _ => None,
                };
                let i = match idx {
                    Some(i) => i as usize,
                    // `chosenPrize[0]` on null / empty: TypeError.
                    None => bail!("TypeError"),
                };
                if g.st.players[p].prize_face_up[i] {
                    bail!("CANNOT_USE_POWER");
                }
                g.st.players[p].prize_public[i] = true;
                g.st.players[p].prize_face_up[i] = true;
                if let Effect::Attack { damage, .. } = g.e_mut(atk) {
                    *damage += 80;
                }
                Ok(())
            })();
            g.release_fx(atk);
            r
        }
        _ => Ok(()),
    }
}
