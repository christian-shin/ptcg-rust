//! Archaludon ex (SSP 130): Assemble Alloy — when you play this Pokémon from
//! your hand to evolve, you may attach 2 Basic [M] Energy from your discard
//! pile to your [M] Pokémon in any way you like. Metal Defender — 220;
//! during your opponent's next turn this Pokémon has no Weakness.
//!
//! Twinleaf: fires on any EvolveEffect for this card (Rare Candy included)
//! when the discard holds an Energy named "Metal Energy"; blockedTo (non-[M]
//! printed types) is taken before the evolution; the ability-lock probe runs
//! after it. AttachEnergyPrompt from the discard (min 1 since phase 4b R7E: "up to
//! 2" in an Ability takes at least 1, rulings 1853/1778; it was min 0; max 2,
//! no cancel),
//! then one MOVE_CARDS per transfer; no shuffle. Metal Defender reduces a
//! ThisPokemonHasNoWeaknessDuringOpponentsNextTurnEffect.
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "Archaludonex", mask: mask(&[k::EVOLVE, k::ATTACK]), reduce, resume: Some(resume), coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    if let Effect::Evolve { p, card, .. } = *g.e(e) {
        if card != me {
            return Ok(());
        }
        let p = p as usize;
        let has = g.st.players[p].discard.iter().any(|c| {
            let d = g.st.cdef(c);
            d.is_energy() && d.name == "Metal Energy"
        });
        if !has {
            return Ok(());
        }
        // blockedTo as a bitmask (bit 0 = Active, 1 + i = Bench i).
        let mut bits = 0i32;
        for (_, c, t) in for_each_pokemon(g, p, PlayerType::BottomPlayer).iter().copied() {
            if !g.st.cdef(c).card_type.contains(&ct::METAL) {
                let bit = if t.slot == SlotType::Active { 0 } else { 1 + t.index as i32 };
                bits |= 1 << bit;
            }
        }
        if is_ability_blocked(g, p, me, None) {
            return Ok(());
        }
        let mut f = CardFrame::at(1);
        f.a[0] = p as i32;
        f.a[1] = bits;
        confirmation_prompt(g, p, "WANT_TO_USE_ABILITY", Cont::Card { card: me, frame: f });
        return Ok(());
    }

    if was_attack_used(g, e, 0, me) {
        let b = match *g.e(e) {
            Effect::Attack { p, opp, attack, source, .. } => AtkBase { attack_effect: e, player: p, opponent: opp, attack, source, target: source },
            _ => return Ok(()),
        };
        g.run_fx(Effect::ThisPokemonHasNoWeakness { b })?;
    }
    Ok(())
}

fn resume(g: &mut Game, me: CardId, f: CardFrame, results: &[Res]) -> R {
    let first = results.first().copied().unwrap_or(Res::Null);
    let p = f.a[0] as usize;
    match f.stage {
        1 => {
            if !first.as_bool() {
                return Ok(());
            }
            let mut o = AttachOpts::new(g.st.players[p].discard.len() as u8);
            o.allow_cancel = false;
            o.min = 1;
            o.max = 2;
            for bit in 0..9 {
                if f.a[1] & (1 << bit) != 0 {
                    let t = if bit == 0 {
                        CardTarget::new(PlayerType::BottomPlayer, SlotType::Active, 0)
                    } else {
                        CardTarget::new(PlayerType::BottomPlayer, SlotType::Bench, (bit - 1) as u8)
                    };
                    o.blocked_to.push(t);
                }
            }
            let mut slots = SVec::new();
            slots.push(SlotType::Active as u8);
            slots.push(SlotType::Bench as u8);
            let filter = Filter { super_type: Some(SuperType::Energy as u8), energy_type: Some(EnergyType::Basic as u8), name: Some("Metal Energy"), ..Filter::none() };
            let mut nf = CardFrame::at(2);
            nf.a[0] = p as i32;
            let id = g.player_id(p);
            g.prompt(
                id,
                "ATTACH_ENERGY_TO_ACTIVE",
                PromptKind::AttachEnergy { cards: ListRef::Discard(p as u8), player_type: PlayerType::BottomPlayer, slots, filter, o },
                Cont::Card { card: me, frame: nf },
            );
            Ok(())
        }
        2 => {
            let transfers: SVec<(CardTarget, CardId), 16> = match first {
                Res::Attach(t) => t,
                _ => SVec::new(),
            };
            for (to, c) in transfers.iter().copied() {
                let target = get_target(&g.st, p, to)?;
                move_cards(g, ListRef::Discard(p as u8), target.list(), &[c], me)?;
            }
            Ok(())
        }
        _ => Ok(()),
    }
}
