//! Annihilape (M5 / PBL): Durable Body — if this Pokémon would be Knocked Out
//! by damage from an attack, flip a coin; heads, it survives with 10 HP.
//! Ghostly Blow — 100, place 5 damage counters on 1 of the opponent's
//! Benched Pokémon.
//!
//! Fixed (phase 4b, X1-1): the coin's callback used to set
//! `surviveOnTenHPReason` after the flip's wait prompt, i.e. after the
//! PutDamageEffect was already applied, so the flip happened but never saved
//! the Pokémon (and the would-KO test ignored the damage already on it). The
//! flip is now read right away (SURVIVE_ON_TEN_ON_COIN_FLIP). Also fixed: when the code runs
//! for a copycat (a copied Ghostly Blow's session), `IS_ABILITY_BLOCKED` is
//! true for the copycat, so Durable Body no longer applies to it (it used to
//! throw here for a copycat whose card has no powers).
//!
//! Fixed (phase 4b, R2): Ghostly Blow placed its counters with a
//! PlaceDamageCountersEffect (an Ability effect), which Mist Energy, Empoleon
//! ex and Skeledirge do not prevent; it now uses a PutCountersEffect (an
//! effect of the attack) on the chosen Benched Pokémon.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl {
    class: "Annihilape@Annihilape M5",
    mask: mask(&[k::PUT_DAMAGE, k::AFTER_ATTACK]),
    reduce,
    resume: Some(resume),
    coin: None,
    can_play: None,
};

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if let Effect::PutDamage { b, .. } = *g.e(e) {
        let t = b.target;
        if g.st.slot(t.p as usize, t.s).cards.contains(me) && g.st.slot_pokemon(t.p as usize, t.s) == Some(me) {
            let owner = t.p as usize;
            if is_ability_blocked(g, owner, me, None) {
                return Ok(());
            }
            crate::prefabs::survive_on_ten_on_coin_flip(g, e, owner)?;
        }
    }

    if after_attack_used(g, e, 0, me) {
        let e = real_attack(g, e);
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
        f.e[0] = e;
        g.retain_fx(e);
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

fn resume(g: &mut Game, _me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let atk = f.e[0];
    let picked: Vec<SlotRef> = results.first().map(|r| r.slots().to_vec()).unwrap_or_default();
    let r = (|| -> R {
        let dest = match picked.first() {
            Some(d) => *d,
            None => return Ok(()),
        };
        let (p, opp, attack, source) = match *g.e(atk) {
            Effect::Attack { p, opp, attack, source, .. } => (p, opp, attack, source),
            _ => return Ok(()),
        };
        let b = AtkBase { attack_effect: atk, player: p, opponent: opp, attack, source, target: dest };
        g.run_fx(Effect::PutCounters { b, damage: 50 })?;
        Ok(())
    })();
    g.release_fx(atk);
    r
}
