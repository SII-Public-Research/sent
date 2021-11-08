# Introduction
Driver for the Protocol SENT, written in the Rust programming language. 
The example uses a NUCLEO STM32F103RB so we used the crate stm32f1xx_hal.

# Status

This driver allows to know the values sent by a sensor with the SENT protocol : the status nibble (4 bits), the data nibbles (6 * 4 bits) and the crc nibble (1 bit). At each falling edge, the value of TIM3 is stocked by a DMA request, using input capture mode. With these times data and a mathematic treatement, we calculate the value of the data sent by the sensor. A function allows to check the crc to validate the frame received.

Two functions allow to manage the dma (enable/disable) to choose when capture the data.

# Usage
Include this crate in your Cargo project by adding the following to Cargo.toml:

[dependencies]
sent-driver = "0.1.0"

# Documentation
Please refer to this link to understand SENT protocol : https://www.renesas.com/us/en/document/whp/tutorial-digital-sent-interface-zssc416xzssc417x

Please refer to the stm32f1xx_hal librairie (docs + examples): https://crates.io/crates/stm32f1xx-hal

# License
This project is open source software, licensed under the terms of the Zero Clause BSD License (0BSD, for short). This basically means you can do anything with the software, without any restrictions, but you can't hold the authors liable for problems.

See LICENSE.md for full details.