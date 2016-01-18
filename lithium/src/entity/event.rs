// Server / Client Side Entity Events -----------------------------------------
pub enum EntityEvent {
    Tick(u8, f32),
    Created(u8),
    Destroyed(u8),
    Flags(u8)
}

