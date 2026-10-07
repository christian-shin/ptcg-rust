//! Illumise (TWM): Slowing Perfume — only if you go second, during your
//! first turn: shuffle 1 of your opponent's Benched Pokémon and all attached
//! cards into their deck. Glide — 30.
//!
//! Twinleaf: legal only on `state.turn == 2` (throws CANNOT_USE_ATTACK
//! otherwise). Fixed (R1-9): it shuffled the opponent's *Active* Pokémon
//! (and ran `clearEffects()` on the vacated Active slot); it now does nothing
//! with an empty opposing Bench, otherwise the attacker chooses 1 Benched
//! Pokémon (ChoosePokemonPrompt, min 1, max 1, no cancel), MOVE_CARDS moves
//! the whole stack into the opponent's deck, then the opponent's deck
//! shuffle is prompted (like Sylveon ex's Angelite).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Illumise@TWM", mask: mask(&[k::ATTACK, k::AFTER_ATTACK]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    // Slowing Perfume can only be used on your first turn: refused before the attack does anything.
    if was_attack_used(g, e, 0, me) && g.st.turn != 2 {
        bail!("CANNOT_USE_ATTACK");
    }
    if !after_attack_used(g, e, 0, me) {
        return Ok(());
    }
    let e = real_attack(g, e);
    let (p, o) = match *g.e(e) {
        Effect::Attack { p, opp, .. } => (p as usize, opp as usize),
        _ => return Ok(()),
    };
    let has_bench = g.st.players[o].bench.iter().any(|s| !g.st.players[o].slots[*s as usize].cards.is_empty());
    if !has_bench {
        return Ok(());
    }
    let mut slots = SVec::new();
    slots.push(SlotType::Bench as u8);
    let mut f = CardFrame::at(1);
    f.a[0] = p as i32;
    if let Effect::Attack { attack, .. } = *g.e(e) {
        f.a[1] = crate::prefabs::pack_attack(attack);
    }
    let id = g.player_id(p);
    g.prompt(
        id,
        "CHOOSE_POKEMON_TO_SHUFFLE",
        PromptKind::ChoosePokemon { player_type: PlayerType::TopPlayer, slots, min: 1, max: 1, allow_cancel: false, blocked: SVec::new() },
        Cont::Card { card: me, frame: f },
    );
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let p = f.a[0] as usize;
    let o = 1 - p;
    let targets: Vec<SlotRef> = results.first().map(|r| r.slots().to_vec()).unwrap_or_default();
    for t in targets {
        // An effect of the attack on that Pokémon: Mist Energy and the like prevent it (R7F-17, ruling 1843).
        if crate::prefabs::attack_effect_prevented_on(g, p, o, f.a[1], t)? {
            continue;
        }
        move_pokemon_off_board(g, t, ListRef::Deck(o as u8), me)?;
    }
    let id = g.player_id(o);
    g.prompt(id, "", PromptKind::ShuffleDeck, Cont::ShuffleApplyNoWait { p: o as u8 });
    Ok(())
}
