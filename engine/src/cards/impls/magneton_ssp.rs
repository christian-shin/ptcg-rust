//! Magneton (SSP): Overvolt Discharge — once during your turn, you may attach
//! up to 3 Basic Energy cards from your discard pile to your [L] Pokémon in
//! any way you like. If you use this Ability, this Pokémon is Knocked Out.
//! Electric Ball — 40.
//!
//! Twinleaf: throws CANNOT_USE_POWER without a basic Energy in the discard;
//! a non-cancellable AttachEnergyPrompt (discard → Bench/Active, basic
//! Energy, min 1 (phase 4b R7E: "up to 3" in an Ability takes at least 1,
//! rulings 1853/1778; it was min 0) max 3, non-[L] Pokémon blocked); MOVE_CARDS each transfer,
//! then `damage += 999` on this card's slot. No once-per-turn marker, no
//! ABILITY_USED.
//!
//! Fixed (phase 4b, R2): with no transfer (0 chosen) Magneton was not Knocked
//! Out; the text says it is Knocked Out whenever the Ability is used.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Magneton@SSP", mask: mask(&[k::POWER]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if !was_power_used(g, e, 0, me) {
        return Ok(());
    }
    let p = match *g.e(e) {
        Effect::Power { p, .. } => p as usize,
        _ => return Ok(()),
    };
    let has = g.st.players[p].discard.iter().any(|c| {
        let d = g.st.cdef(c);
        d.is_energy() && d.energy_type == EnergyType::Basic as u8
    });
    if !has {
        bail!("CANNOT_USE_POWER");
    }
    let mut o = AttachOpts::new(g.st.players[p].discard.len() as u8);
    o.allow_cancel = false;
    o.min = 1;
    o.max = 3;
    for (_, c, t) in for_each_pokemon(g, p, PlayerType::BottomPlayer).iter().copied() {
        if !g.st.cdef(c).card_type.contains(&ct::LIGHTNING) {
            o.blocked_to.push(t);
        }
    }
    let filter = Filter { super_type: Some(SuperType::Energy as u8), energy_type: Some(EnergyType::Basic as u8), ..Filter::none() };
    let mut slots = SVec::new();
    slots.push(SlotType::Bench as u8);
    slots.push(SlotType::Active as u8);
    let mut f = CardFrame::at(1);
    f.a[0] = p as i32;
    let id = g.player_id(p);
    g.prompt(
        id,
        "ATTACH_ENERGY_CARDS",
        PromptKind::AttachEnergy { cards: ListRef::Discard(p as u8), player_type: PlayerType::BottomPlayer, slots, filter, o },
        Cont::Card { card: me, frame: f },
    );
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    if f.stage != 1 {
        return Ok(());
    }
    let p = f.a[0] as usize;
    let transfers: SVec<(CardTarget, CardId), 16> = match results.first() {
        Some(Res::Attach(t)) => *t,
        _ => SVec::new(),
    };
    for (to, c) in transfers.iter().copied() {
        let target = get_target(&g.st, p, to)?;
        move_cards(g, ListRef::Discard(p as u8), target.list(), &[c], me)?;
    }
    for (s, _, _) in for_each_pokemon(g, p, PlayerType::BottomPlayer).iter().copied() {
        if g.st.slot_pokemon(p, s) == Some(me) {
            g.st.players[p].slots[s as usize].damage += 999;
        }
    }
    Ok(())
}
