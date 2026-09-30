//! Annihilape (M5 / PBL): Durable Body — if this Pokémon would be Knocked Out
//! by damage from an attack, flip a coin; heads, it survives with 10 HP.
//! Ghostly Blow — 100, place 5 damage counters on 1 of the opponent's
//! Benched Pokémon.
//!
//! Twinleaf quirk kept: the coin's callback sets `surviveOnTenHPReason`
//! after the flip's wait prompt, i.e. after the PutDamageEffect was already
//! applied, so the flip happens but never saves the Pokémon. On heads the
//! callback reads `this.powers[0].name`: when the code runs for a copycat
//! (a copied Ghostly Blow's session) whose card has no powers, that throws.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl {
    class: "Annihilape@Annihilape M5",
    mask: mask(&[k::PUT_DAMAGE, k::ATTACK]),
    reduce,
    resume: Some(resume),
    coin: Some(coin),
    can_play: None,
};

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if let Effect::PutDamage { b, damage, .. } = *g.e(e) {
        let t = b.target;
        if g.st.slot(t.p as usize, t.s).cards.contains(me) && g.st.slot_pokemon(t.p as usize, t.s) == Some(me) {
            let owner = t.p as usize;
            if is_ability_blocked(g, owner, me, None) {
                return Ok(());
            }
            let hp = crate::engine::check::check_hp(g, owner, t.s)?;
            if damage >= hp {
                g.coin_flip(owner, CoinCb::Card { card: me, frame: CardFrame::at(2) })?;
                return Ok(());
            }
        }
    }

    if was_attack_used(g, e, 0, me) {
        let p = match *g.e(e) {
            Effect::Attack { p, .. } => p as usize,
            _ => return Ok(()),
        };
        let o = 1 - p;
        let pl = &g.st.players[o];
        if !pl.bench.iter().any(|b| !pl.slots[*b as usize].cards.is_empty()) {
            return Ok(());
        }
        let mut slots = SVec::new();
        slots.push(SlotType::Bench as u8);
        let mut f = CardFrame::at(1);
        f.a[0] = p as i32;
        let id = g.player_id(p);
        g.prompt(
            id,
            "CHOOSE_POKEMON_TO_DAMAGE",
            PromptKind::ChoosePokemon { player_type: PlayerType::TopPlayer, slots, min: 1, max: 1, allow_cancel: false, blocked: SVec::new() },
            Cont::Card { card: me, frame: f },
        );
    }
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let p = f.a[0] as usize;
    let picked: Vec<SlotRef> = results.first().map(|r| r.slots().to_vec()).unwrap_or_default();
    let dest = match picked.first() {
        Some(d) => *d,
        None => return Ok(()),
    };
    g.run_fx(Effect::PlaceDamageCounters { p: p as u8, target: dest, damage: 50, source: me })?;
    Ok(())
}

fn coin(g: &mut Game, me: CardId, _f: CardFrame, heads: bool) -> R {
    // `effect.surviveOnTenHPReason = this.powers[0].name` (no rules effect).
    if heads && g.st.cdef(me).powers.is_empty() {
        bail!("Cannot read properties of undefined (reading 'name')");
    }
    Ok(())
}
