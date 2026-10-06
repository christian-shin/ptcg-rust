//! Minccino (TEF): Beat — 10. Cleaning Up — discard up to 2 Pokémon Tools
//! from your opponent's Pokémon.
//!
//! Twinleaf has two `Minccino` classes; this port is bound to TEF.
//! The Cleaning Up code is on attack 1 (fixed in phase 4b: it was attached to
//! attack 0, Beat). Cleaning Up does nothing when no opposing Pokémon has a
//! Tool (fixed in R1-12: it used to throw CANNOT_PLAY_THIS_CARD, so the attack
//! was not offered), otherwise the AttackEffect is prevented (no damage; the
//! attack has none).
//!
//! Fixed (phase 4b, F1; rulings 1721, 1843): the choice is over the Tools, not
//! the Pokémon. One DiscardEnergyPrompt with a Pokémon Tool filter over every
//! Tool attached to the opponent's Pokémon (min 0, max min(2, Tools in play),
//! not cancellable) replaces "choose 1-2 Pokémon, then Tools on one holding
//! several". Each chosen Tool is checked per Pokémon for an effect that
//! prevents the attack's effects (Mist Energy, ruling 1843).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Minccino@TEF", mask: mask(&[k::ATTACK]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if !was_attack_used(g, e, 1, me) {
        return Ok(());
    }
    let (p, opp, source) = match *g.e(e) {
        Effect::Attack { p, opp, source, .. } => (p as usize, opp as usize, source),
        _ => return Ok(()),
    };
    let mut tools_in_play = 0usize;
    for (s, _, _) in for_each_pokemon(g, opp, PlayerType::TopPlayer).iter().copied() {
        tools_in_play += g.st.slot(opp, s).tools.len();
    }
    if tools_in_play == 0 {
        return Ok(());
    }
    g.set_prevent(e, true);
    let mut slots = SVec::new();
    slots.push(SlotType::Active as u8);
    slots.push(SlotType::Bench as u8);
    let mut f = CardFrame::at(1);
    f.a[0] = p as i32;
    if let Effect::Attack { attack, .. } = *g.e(e) {
        f.a[1] = crate::prefabs::pack_attack(attack);
    }
    f.l[0] = source.p;
    f.l[1] = source.s;
    let filter = Filter { super_type: Some(SuperType::Trainer as u8), trainer_type: Some(TrainerType::Tool as u8), ..Filter::none() };
    let o = MoveOpts { allow_cancel: false, min: 0, max: Some(tools_in_play.min(2) as u8), ..Default::default() };
    let id = g.player_id(p);
    g.prompt(
        id,
        "CHOOSE_CARD_TO_DISCARD",
        PromptKind::DiscardEnergy { player_type: PlayerType::TopPlayer, slots, filter, o },
        Cont::Card { card: me, frame: f },
    );
    Ok(())
}

fn source_card(g: &Game, f: &CardFrame) -> CardId {
    g.st.slot_pokemon(f.l[0] as usize, f.l[1]).unwrap_or(NO_CARD)
}

fn resume(g: &mut Game, _me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let p = f.a[0] as usize;
    let transfers = match results.first().copied() {
        Some(Res::CardsFrom(t)) => t,
        _ => return Ok(()),
    };
    for (from, card) in transfers.iter().copied() {
        let t = get_target(&g.st, p, from)?;
        let owner = t.p as usize;
        // An effect of the attack on that Pokémon: Mist Energy and the like prevent it (R7F-17, ruling 1843).
        if crate::prefabs::attack_effect_prevented_on(g, p, owner, f.a[1], t)? {
            continue;
        }
        let sc = source_card(g, &f);
        move_cards(g, ListRef::Slot(owner as u8, t.s), ListRef::Discard(owner as u8), &[card], sc)?;
    }
    Ok(())
}
