#[test]
fn sizes() {
    use std::mem::size_of;
    println!("Effect {} EffSlot {} PromptRec {} PromptItem {} Cont {} Slot {} Player {} State {} Game {}",
        size_of::<ptcg::effects::Effect>(), size_of::<ptcg::game::EffSlot>(), size_of::<ptcg::prompts::PromptRec>(),
        size_of::<ptcg::game::PromptItem>(), size_of::<ptcg::game::Cont>(), size_of::<ptcg::state::Slot>(),
        size_of::<ptcg::state::Player>(), size_of::<ptcg::state::State>(), size_of::<ptcg::game::Game>());
}
