//! Survival Brace (TWM, ACE SPEC tool): if the Pokémon this card is attached
//! to has full HP and would be Knocked Out by damage from an opponent's
//! attack, that Pokémon is not Knocked Out and its remaining HP becomes 10
//! instead. Then, discard this card.
//!
//! Twinleaf: on a PutDamageEffect whose target holds this tool, during the
//! attack phase and not from the owner's own Pokémon (phase 4b: it used to
//! trigger on any PutDamageEffect), unless the tool is blocked, when the slot
//! has no damage and `effect.damage >=` its HP (CheckHpEffect by the owner):
//! sets `surviveOnTenHPReason` and discards the tool from every slot of the
//! owner holding it. The core then caps the damage at HP - 10 whenever it
//! reached HP (phase 4b: exactly lethal damage used to Knock Out anyway).
use crate::cards::prelude::*;

pub static IMPL: CardImpl = CardImpl { class: "SurvivalCast", mask: mask(&[k::PUT_DAMAGE]), reduce, resume: None, coin: None, can_play: None };

fn reduce(g: &mut Game, me: CardId, e: EffId) -> R {
    let (t, damage) = match *g.e(e) {
        Effect::PutDamage { b, damage, .. } => (b.target, damage),
        _ => return Ok(()),
    };
    if !g.st.slot(t.p as usize, t.s).tools.contains(me) {
        return Ok(());
    }
    let owner = t.p as usize;
    // Only damage from an attack by the opponent's Pokémon
    let attacker = match *g.e(e) {
        Effect::PutDamage { b, .. } => b.player as usize,
        _ => return Ok(()),
    };
    if attacker == owner || g.st.phase != GamePhase::Attack {
        return Ok(());
    }
    if is_tool_blocked(g, owner, me) {
        return Ok(());
    }
    if g.st.slot(owner, t.s).damage != 0 {
        return Ok(());
    }
    let hp = crate::engine::check::check_hp(g, owner, t.s)?;
    if damage < hp {
        return Ok(());
    }
    if let Effect::PutDamage { survive_on_ten_hp, .. } = g.e_mut(e) {
        *survive_on_ten_hp = true;
    }
    for (s, _, _) in for_each_pokemon(g, owner, PlayerType::BottomPlayer).iter().copied() {
        if g.st.slot(owner, s).tools.contains(me) {
            move_cards(g, ListRef::Slot(owner as u8, s), ListRef::Discard(owner as u8), &[me], me)?;
        }
    }
    Ok(())
}
