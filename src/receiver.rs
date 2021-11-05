impl<DELAY, PIN> crate::Sent<DELAY, PIN> {
    pub fn receive_frame() -> (u8, [u8; 6]) {
        //capture front descendant
        //calcule temps entre deux captures
        //verificarion t_sync
        //capture du reste de la frame

        //retour de la frame
        (32, [1, 2, 3, 4, 5, 6])
    }
}
