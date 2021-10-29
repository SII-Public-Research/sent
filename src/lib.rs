#![no_std]

//use embedded_hal::{digital::v2::InputPin, blocking::delay::DelayUs};

mod receiver;
mod transmitter;

#[derive(Debug)]
pub struct Sent<DELAY, PIN> {
    pub delay: DELAY,
    pub pin: PIN,
    pub t_frame: u32,
    pub t_tick: u32,
    pub t_sync: u32,
    pub t_offset: u32,
    pub nb_nibbles: u32,
}

impl<DELAY, PIN> Sent<DELAY, PIN> {
    pub fn new(
        delay: DELAY,
        pin: PIN,
        t_frame: u32,
        t_tick: u32,
        t_sync: u32,
        t_offset: u32,
        nb_nibbles: u32,
    ) -> Self {
        Self {
            delay,
            pin,
            t_frame,
            t_tick,
            t_sync,
            t_offset,
            nb_nibbles,
        }
    }

    pub fn new_default(delay: DELAY, pin: PIN) -> Self {
        Self::new(delay, pin, 5000, 15, 56 * 15, 12 * 15, 6)
    }
}

pub fn calcul_checksum(status: &u8, data: &[u8; 6]) -> u8 {
    let crc_data: u8 = status + data.iter().sum::<u8>();

    0xF - (crc_data & 0xF)
}
