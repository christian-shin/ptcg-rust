//! Alolan Exeggutor ex (SSP, Tera): Tropical Fever — 150; attach any number
//! of Basic Energy from your hand to your Pokémon in any way. Swinging
//! Sphene — flip a coin; heads: Knock Out the opponent's Active if it is
//! Basic; tails: Knock Out 1 of the opponent's Benched Pokémon.
//!
//! Twinleaf: tails opens a ChoosePokemonPrompt over the opponent's Bench with
//! the non-Basic Benched Pokémon blocked, and does nothing when no Benched
//! Basic exists (phase 4b fix: `blocked.push()` pushed nothing, so any Benched
//! Pokémon could be Knocked Out, and an empty Bench left the prompt
//! unanswerable). Tropical Fever's AttachEnergyPrompt has the default `max` =
//! hand size.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl {
    class: "AlolanExeggutorex",
    mask: mask(&[k::ATTACK, k::PUT_DAMAGE, k::AFTER_ATTACK]),
    reduce,
    resume: Some(resume),
    coin: Some(coin),
    can_play: None,
};

fn ko(g: &mut Game, atk: EffId, target: SlotRef) -> R {
    if let Effect::Attack { p, opp, attack, source, .. } = *g.e(atk) {
        let b = AtkBase { attack_effect: atk, player: p, opponent: opp, attack, source, target };
        g.run_fx(Effect::KnockOutOpponent { b, knocked_out: false, prize_count: 0 })?;
    }
    Ok(())
}

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    tera_rule(g, e, me);
    if after_attack_used(g, e, 0, me) {
        let e = real_attack(g, e);
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        let has = g.st.players[p].hand.iter().any(|c| {
            let d = g.st.cdef(c);
            d.is_energy() && d.energy_type == EnergyType::Basic as u8
        });
        if !has {
            return Ok(());
        }
        let mut slots = SVec::new();
        slots.push(SlotType::Bench as u8);
        slots.push(SlotType::Active as u8);
        let mut o = AttachOpts::new(g.st.players[p].hand.len().min(255) as u8);
        o.allow_cancel = false;
        let mut f = CardFrame::at(1);
        f.a[0] = p as i32;
        let id = g.player_id(p);
        g.prompt(
            id,
            "ATTACH_ENERGY_CARDS",
            PromptKind::AttachEnergy {
                cards: ListRef::Hand(p as u8),
                player_type: PlayerType::BottomPlayer,
                slots,
                filter: Filter { super_type: Some(SuperType::Energy as u8), energy_type: Some(EnergyType::Basic as u8), ..Filter::none() },
                o,
            },
            Cont::Card { card: me, frame: f },
        );
    }
    if after_attack_used(g, e, 1, me) {
        let e = real_attack(g, e);
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        g.retain_fx(e);
        let mut f = CardFrame::at(0);
        f.a[0] = p as i32;
        f.e[0] = e;
        if let Err(err) = g.coin_flip(p, CoinCb::Card { card: me, frame: f }) {
            g.release_fx(e);
            return Err(err);
        }
    }
    Ok(())
}

fn coin(g: &mut Game, me: CardId, f: CardFrame, heads: bool) -> R {
    let p = f.a[0] as usize;
    let atk = f.e[0];
    let o = 1 - p;
    if heads {
        let r = (|| -> R {
            let a = g.st.players[o].active;
            if let Some(c) = g.st.slot_pokemon(o, a) {
                if g.st.cdef(c).stage != Stage::Basic as u8 {
                    return Ok(());
                }
            }
            ko(g, atk, SlotRef::new(o, a))
        })();
        g.release_fx(atk);
        return r;
    }
    // Tails: only Benched Basic Pokémon can be Knocked Out (phase 4b fix).
    let mut blocked: TargetList = SVec::new();
    let mut basics = 0;
    for (_, c, t) in for_each_pokemon(g, o, PlayerType::TopPlayer).iter().copied() {
        if t.slot != SlotType::Bench {
            continue;
        }
        if g.st.cdef(c).stage == Stage::Basic as u8 {
            basics += 1;
        } else {
            blocked.push(t);
        }
    }
    if basics == 0 {
        g.release_fx(atk);
        return Ok(());
    }
    let mut slots = SVec::new();
    slots.push(SlotType::Bench as u8);
    let mut nf = CardFrame::at(2);
    nf.a[0] = p as i32;
    nf.e[0] = atk;
    let id = g.player_id(p);
    g.prompt(
        id,
        "CHOOSE_POKEMON_TO_DAMAGE",
        PromptKind::ChoosePokemon { player_type: PlayerType::TopPlayer, slots, min: 1, max: 1, allow_cancel: false, blocked },
        Cont::Card { card: me, frame: nf },
    );
    Ok(())
}

fn resume(g: &mut Game, _me: CardId, f: CardFrame, results: &[Res]) -> R {
    let p = f.a[0] as usize;
    match f.stage {
        1 => {
            let transfers: SVec<(CardTarget, CardId), 64> = match results.first() {
                Some(Res::Attach(t)) => *t,
                _ => SVec::new(),
            };
            for (to, c) in transfers.iter().copied() {
                let target = get_target(&g.st, p, to)?;
                g.run_fx(Effect::AttachEnergy { p: p as u8, card: c, target })?;
            }
            Ok(())
        }
        2 => {
            let atk = f.e[0];
            let targets: Vec<SlotRef> = results.first().map(|r| r.slots().to_vec()).unwrap_or_default();
            let mut r = Ok(());
            for t in targets {
                r = ko(g, atk, t);
                if r.is_err() {
                    break;
                }
            }
            g.release_fx(atk);
            r
        }
        _ => Ok(()),
    }
}
