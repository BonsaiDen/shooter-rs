// Server / Client Side Entity Events -----------------------------------------
#[derive(Debug)]
pub enum EntityEvent {
    Tick(u8, f32),
    Created(u8),
    Destroyed(u8),
    Hide(u8),
    Show(u8),
    Flags(u8)
}

