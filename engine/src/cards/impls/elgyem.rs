//! Elgyem (BLK / SV11B): Slight Shift - move an Energy from 1 of your
//! opponent's Pokémon to another of their Pokémon. Beam - 40.
//!
//! Twinleaf: MOVE_AN_ENERGY_FROM_OPPONENTS_POKEMON_TO_ANOTHER: needs an
//! Energy card on some opponent Pokémon and at least 2 Pokémon in play, then a
//! mandatory MoveEnergyPrompt (1 transfer); each transfer is a
//! MoveOpponentEnergyEffect (an attack effect that Mist Energy & co. block).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Elgyem", mask: mask(&[k::ATTACK]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if !was_attack_used(g, e, 0, me) {
        return Ok(());
    }
    let (p, o) = match *g.e(e) {
        Effect::Attack { p, opp, .. } => (p as usize, opp as usize),
        _ => return Ok(()),
    };
    let mons = for_each_pokemon(g, o, PlayerType::TopPlayer);
    let has_energy = mons.iter().any(|(s, _, _)| g.st.slot(o, *s).cards.iter().any(|c| g.st.cdef(c).is_energy()));
    if !has_energy || mons.len() <= 1 {
        return Ok(());
    }
    let mut slots = SVec::new();
    slots.push(SlotType::Active as u8);
    slots.push(SlotType::Bench as u8);
    let opts = MoveOpts { allow_cancel: false, min: 1, max: Some(1), ..Default::default() };
    g.retain_fx(e);
    let mut f = CardFrame::at(1);
    f.e[0] = e;
    let id = g.player_id(p);
    g.prompt(
        id,
        "MOVE_ENERGY_CARDS",
        PromptKind::MoveEnergy { player_type: PlayerType::TopPlayer, slots, filter: Filter::super_type(SuperType::Energy), o: opts },
        Cont::Card { card: me, frame: f },
    );
    Ok(())
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
        if let Some(Res::Transfers(ts)) = results.first() {
            for (from, to, card) in ts.iter() {
                let src = get_target(&g.st, p as usize, *from)?;
                let dst = get_target(&g.st, p as usize, *to)?;
                let b = AtkBase { attack_effect: atk, player: p, opponent: opp, attack, source, target: src };
                g.run_fx(Effect::MoveOpponentEnergy { b, card: *card, destination: dst })?; // queued until after the damage (R7A)
            }
        }
        Ok(())
    })();
    g.release_fx(atk);
    r
}
