use gte_w65c02s::System;
pub trait Bus: System {
    fn clear_cycles(&mut self) -> u8;
}
