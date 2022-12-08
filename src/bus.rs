use crate::Pins;

pub trait Bus {
    fn tick(&mut self, pins: &mut Pins);
}
