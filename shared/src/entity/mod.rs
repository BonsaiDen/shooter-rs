// Internal Dependencies ------------------------------------------------------
mod entity;
mod input;
mod state;
pub mod traits;


// Re-Exports -----------------------------------------------------------------
pub use entity::entity::Entity as Entity;
pub use entity::input::EntityInput as Input;
pub use entity::state::EntityState as State;


// Utilities ------------------------------------------------------------------
pub fn tick_is_more_recent(a: u8, b: u8) -> bool {
    (a > b) && (a - b <= 255 / 2) || (b > a) && (b - a > 255 / 2)
}

pub fn tick_entity<F>(
    state: &mut state::EntityState,
    base_state: &mut state::EntityState,
    last_state: &mut state::EntityState,
    remote_state: &mut Option<(u8, state::EntityState)>,
    input_states: &mut Vec<input::EntityInput>,
    set_base_state: bool,
    apply_inputs: F

) where F: Fn(state::EntityState, &mut Vec<input::EntityInput>) -> state::EntityState {

    // Check if we have a remote state
    if let Some((remote_tick, remote_state)) = remote_state.take() {

        // Set the current state as the last state
        *last_state = *state;

        // Take over the remote state as the new base
        *base_state = remote_state;
        *state = remote_state;

        // Drop all inputs confirmed by the remote so the remaining ones
        // get applied on top of the new base state
        input_states.retain(|input| {
            tick_is_more_recent(input.tick, remote_tick)
        });

    // Otherwise reset the local state and re-apply the inputs on top of it
    } else {
        *last_state = *state;
        *state = *base_state;
    }

    // Apply unconfirmed inputs on top of last state confirmed by the server
    *state = apply_inputs(*base_state, input_states);

    // Use the newly calculated state as the base
    if set_base_state {
        *base_state = *state;
        input_states.clear();
    }

}
