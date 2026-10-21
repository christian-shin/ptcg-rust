#[test]
fn sizes() {
    use std::mem::size_of;
    println!("Effect {} EffSlot {} PromptRec {} PromptItem {} Cont {} Slot {} Player {} State {} Game {}",
        size_of::<ptcg::effects::Effect>(), size_of::<ptcg::game::EffSlot>(), size_of::<ptcg::prompts::PromptRec>(),
        size_of::<ptcg::game::PromptItem>(), size_of::<ptcg::game::Cont>(), size_of::<ptcg::state::Slot>(),
        size_of::<ptcg::state::Player>(), size_of::<ptcg::state::State>(), size_of::<ptcg::game::Game>());
    println!("CardInst {} LastingEffect {}", size_of::<ptcg::state::CardInst>(), size_of::<ptcg::state::LastingEffect>());
    // Events batch 8 C7: the lasting effects in the lasting table, `hp` / `hpBonus` and the dead card-object fields gone
    // (Slot 568 -> 256 B, State 17,816 -> 9,712 B).
    // Events batch 9 C6: the string markers gone (Slot 248 -> 176 B, State 9,704 -> 7,920 B).
    assert!(size_of::<ptcg::state::Slot>() <= 192, "Slot grew: {} B", size_of::<ptcg::state::Slot>());
    assert!(size_of::<ptcg::state::CardInst>() <= 16, "CardInst grew: {} B", size_of::<ptcg::state::CardInst>());
    assert!(size_of::<ptcg::state::State>() <= 8192, "State grew: {} B", size_of::<ptcg::state::State>());
    // Events batch 8 C8: the large payloads (the slow path's Energy map, attack list, Weakness / Resistance; the moved
    // damage counters) live in `Game::fxp`; a card list (65 B) stays inline (392 -> 88 B).
    assert!(size_of::<ptcg::effects::Effect>() <= 96, "Effect grew: {} B", size_of::<ptcg::effects::Effect>());
    // The answers of a group's prompts wait in `Game::answers`, not in the prompt (1,040 -> 584 B).
    // C8c: the MoveOpts and the offered Energy in `Game::prompt_payloads` (584 -> 232 B).
    assert!(size_of::<ptcg::prompts::PromptRec>() <= 256, "PromptRec grew: {} B", size_of::<ptcg::prompts::PromptRec>());
}

/// What a search's restore moves (speed batch S0): the whole `Game` (what a copy would move if `Game` were `Copy`; since
/// speed batch S8 it isn't: every copy moves the live parts) and the fixed part of a snapshot, `Game::copy_bytes` of a
/// game with empty stacks: the state and the fixed fields `copy_from` always copies, plus the caches it copies with them
/// (since S8; the derived memo and the order cache's live lists).
/// A game in progress adds its live stack entries (`bench --search` prints the mean and the maximum over the bench's
/// games). A `Snapshot` holds the same bytes plus the declaration index.
#[test]
fn snapshot_sizes() {
    use std::mem::size_of;
    let b = ptcg::game::Game::new(1).copy_bytes();
    println!("Game {} State {} snapshot fixed part {} B + caches {} B", size_of::<ptcg::game::Game>(),
        size_of::<ptcg::state::State>(), b.fields, b.caches);
    // Speed batch S0 (events-10 530d6c8c): Game 64,032 B, State 7,944 B, snapshot 8,238 B copied + 1,212 B cleared. S8
    // (058df6e1 + S8): fields 8,382 B, caches 11,079 B (copied by copy_from and clone since S8).
    assert!(size_of::<ptcg::game::Game>() <= 65_536, "Game grew: {} B", size_of::<ptcg::game::Game>());
    assert!(b.fields <= 8_448, "the fixed part of a snapshot grew: {} B", b.fields);
    assert!(b.caches <= 11_264, "the caches a restore copies grew: {} B", b.caches);
}
