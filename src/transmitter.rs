use embedded_hal::{digital::v2::OutputPin, blocking::delay::DelayUs};

impl<ERR, DELAY: DelayUs<u32>, PIN: OutputPin<Error = ERR>> crate::Sent<DELAY, PIN> {
    pub fn send_frame(&mut self, status: u8, data: [u8; 6], board_offset: u32) {
		// calcul du checksum (fonction)
		// calcul du temps pour status, data, checksum et pause
	    let t_status: u32 = status as u32 * self.t_tick;
        let mut t_data: [u32; 6] = [0; 6];
        for i in 0..self.nb_nibbles as usize {
            t_data[i] = data[i] as u32 * self.t_tick;
        }

        let t_checksum: u32 = crate::calcul_checksum(&status, &data) as u32 * self.t_tick;
        let t_pause: u32 = self.t_frame - (self.t_sync + t_status + t_data.iter().sum::<u32>() + t_checksum + (3 + self.nb_nibbles) * self.t_offset) - 10 * board_offset;
        
        // envoi sync -> status -> data -> checksum -> pause
        self.send_nibble(self.t_sync, false, board_offset);
        self.send_nibble(t_status, true, board_offset);
        for i in 0..self.nb_nibbles as usize {
            self.send_nibble(t_data[i], true, board_offset);
        }

        self.send_nibble(t_checksum, true, board_offset);
        self.send_nibble(t_pause, true, board_offset);
    }

    fn send_nibble(&mut self, nibble_time: u32, use_offset: bool, board_offset: u32) {
        self.pin.set_low().ok();
        self.delay.delay_us(self.t_offset - board_offset);
        self.pin.set_high().ok();
        self.delay.delay_us(nibble_time - if !use_offset{self.t_offset} else {0});
    }
}
