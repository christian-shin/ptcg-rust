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
    assert!(size_of::<ptcg::state::Slot>() <= 256, "Slot grew: {} B", size_of::<ptcg::state::Slot>());
    assert!(size_of::<ptcg::state::CardInst>() <= 16, "CardInst grew: {} B", size_of::<ptcg::state::CardInst>());
    assert!(size_of::<ptcg::state::State>() <= 9728, "State grew: {} B", size_of::<ptcg::state::State>());
    // A MoveCounters event carries up to 32 pairs within the Effect size.
    assert!(size_of::<ptcg::effects::Effect>() <= 392, "Effect grew: {} B", size_of::<ptcg::effects::Effect>());
}
