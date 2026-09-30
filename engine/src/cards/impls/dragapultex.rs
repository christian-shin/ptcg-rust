//! Dragapult ex (TWM): Phantom Dive - 200, and put 6 damage counters on your
//! opponent's Benched Pokémon in any way you like. Tera: no damage from
//! attacks while on the Bench.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl {
    class: "Dragapultex",
    mask: mask(&[k::ATTACK, k::PUT_DAMAGE]),
    reduce,
    resume: Some(resume),
    coin: None,
    can_play: None,
};

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if was_attack_used(g, e, 1, me) {
        put_x_damage_counters_in_any_way_you_like(g, e, 6, me);
    }

    if let Effect::PutDamage { b, .. } = *g.e(e) {
        let t = b.target;
        let slot = g.st.slot(t.p as usize, t.s);
        if slot.cards.contains(me) && g.st.slot_pokemon(t.p as usize, t.s) == Some(me) {
            let pl = b.player as usize;
            let op = 1 - pl;
            // Target is not Active.
            if (t.p as usize == pl && t.s == g.st.players[pl].active) || (t.p as usize == op && t.s == g.st.players[op].active) {
                return Ok(());
            }
            g.set_prevent(e, true);
        }
    }
    Ok(())
}

/// `PUT_X_DAMAGE_COUNTERS_IN_ANY_WAY_YOU_LIKE(x, ..., [SlotType.BENCH])`.
fn put_x_damage_counters_in_any_way_you_like(g: &mut Game, atk: EffId, x: i32, me: CardId) {
    let (p, o) = match *g.e(atk) {
        Effect::Attack { p, opp, .. } => (p as usize, opp as usize),
        _ => return,
    };
    let opl = &g.st.players[o];
    let has_benched = opl.bench.iter().any(|b| !opl.slots[*b as usize].cards.is_empty());
    if !has_benched {
        return;
    }
    let mut max_allowed: SVec<(CardTarget, i32), 16> = SVec::new();
    for t in slot_targets(&g.st, p, PlayerType::TopPlayer, &[SlotType::Active as u8, SlotType::Bench as u8]) {
        max_allowed.push((t, 9999));
    }
    let mut slots = SVec::new();
    slots.push(SlotType::Bench as u8);
    g.retain_fx(atk);
    let mut f = CardFrame::at(1);
    f.e[0] = atk;
    let id = g.player_id(p);
    g.prompt(
        id,
        "CHOOSE_POKEMON_TO_DAMAGE",
        PromptKind::PutDamage {
            player_type: PlayerType::TopPlayer,
            slots,
            damage: 10 * x,
            max_allowed,
            allow_cancel: false,
            blocked: SVec::new(),
            allow_partial: false,
            damage_multiple: 10,
        },
        Cont::Card { card: me, frame: f },
    );
}

fn resume(g: &mut Game, _me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let atk = f.e[0];
    let r = (|| -> R {
        let (p, opp, attack, source) = match *g.e(atk) {
            Effect::Attack { p, opp, attack, source, .. } => (p, opp, attack, source),
            _ => return Ok(()),
        };
        let map: SVec<(CardTarget, i32), 16> = match results.first().copied().unwrap_or(Res::Null) {
            Res::DamageMap(m) => m,
            _ => SVec::new(),
        };
        for (t, damage) in map.iter() {
            let target = get_target(&g.st, p as usize, *t)?;
            let b = AtkBase { attack_effect: atk, player: p, opponent: opp, attack, source, target };
            g.run_fx(Effect::PutCounters { b, damage: *damage })?;
        }
        Ok(())
    })();
    g.release_fx(atk);
    r
}

