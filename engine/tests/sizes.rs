#[test]
fn sizes() {
    use std::mem::size_of;
    println!("Effect {} EffSlot {} PromptRec {} PromptItem {} Cont {} Slot {} Player {} State {} Game {}",
        size_of::<ptcg::effects::Effect>(), size_of::<ptcg::game::EffSlot>(), size_of::<ptcg::prompts::PromptRec>(),
        size_of::<ptcg::game::PromptItem>(), size_of::<ptcg::game::Cont>(), size_of::<ptcg::state::Slot>(),
        size_of::<ptcg::state::Player>(), size_of::<ptcg::state::State>(), size_of::<ptcg::game::Game>());
    // Events batch 6 closeout: the lasting preventions store an index into `passive::LASTING_PREVENTS` (672 -> 608 B).
    assert!(size_of::<ptcg::state::Slot>() <= 608, "Slot grew: {} B", size_of::<ptcg::state::Slot>());
    // A MoveCounters event carries up to 32 pairs within the Effect size.
    assert!(size_of::<ptcg::effects::Effect>() <= 392, "Effect grew: {} B", size_of::<ptcg::effects::Effect>());
}
