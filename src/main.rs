#![no_std]
#![no_main]

mod fmt;
mod buzz;

use buzz::Buzzer;

use cortex_m::prelude::_embedded_hal_Pwm;
use cortex_m_rt::entry;
#[cfg(not(feature = "defmt"))]
use panic_halt as _;
#[cfg(feature = "defmt")]
use {defmt_rtt as _, panic_probe as _};

use embassy_executor::Spawner;
use embassy_stm32::timer::Ch1;
use embassy_stm32::Peri;
use embassy_stm32::gpio::{Level, Output, OutputType, Speed, AnyPin};
use embassy_stm32::peripherals::{TIM1, TIM10};
use embassy_stm32::time::hz;
use embassy_stm32::timer::Channel;
use embassy_stm32::timer::low_level::CountingMode;
use embassy_stm32::timer::simple_pwm::{PwmPin, SimplePwm};
use embassy_time::{Duration, Timer};
use fmt::info;

const AMOGUS: &[(&str, u32)] = &[
    ("C5", 225), ("_", 25),
    ("D#5", 225), ("_", 25),
    ("F5", 225), ("_", 25),
    ("F#5", 225), ("_", 25),
    ("F5", 225), ("_", 25),
    ("D#5", 225), ("_", 25),
    ("C5", 725), ("_", 25),
    ("B#4", 125), ("D5", 125),
    ("C5", 725), ("_", 25),
    ("_", 1000),
    ("C5", 225), ("_", 25),
    ("D#5", 225), ("_", 25),
    ("F5", 225), ("_", 25),
    ("F#5", 225), ("_", 25),
    ("F5", 225), ("_", 25),
    ("D#5", 225), ("_", 25),
    ("F#5", 725), ("_", 150),
    ("F#5", 100), ("_", 25),
    ("F5", 100), ("_", 25),
    ("D#5", 100), ("_", 25),
    ("F#5", 100), ("_", 25),
    ("F5", 100), ("_", 25),
    ("D#5", 100), ("_", 25),
];

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_stm32::init(Default::default());

    let test_data = [
        (255, 0, 0),
        (0, 255, 0),
        (0, 0, 255),
        (255, 0, 0),
        (0, 255, 0),
        (0, 0, 255),
        (255, 255, 255),
        (255, 255, 255)
    ];

    let pwm_pin = PwmPin::new(p.PE9, OutputType::PushPull);
    let mut pwm_driver = SimplePwm::new(p.TIM1, Some(pwm_pin), None, None, None, hz(800_000), CountingMode::EdgeAlignedDown);

    let mut dma = p.DMA2_CH5;

    let mut buzzer = Buzzer::new_ch1(p.TIM10, p.PB8);
    buzzer.set_notes(
        CHORUS_STAYIN_ALIVE
    );
    
    
    
    spawner.spawn(buzzer_task(buzzer)).unwrap();
    
    loop {
        info!("TICK!");
        pwm_driver.waveform_up(
            dma.reborrow(),
            Channel::Ch1,
            &produce_duty_packet8(pwm_driver.get_max_duty(), test_data)
        ).await;
    }
}

#[embassy_executor::task]
async fn buzzer_task(mut buzz: Buzzer<'static, TIM10>) {
    loop {
        info!("TICK!");
        buzz.buzzer_task().await;
    }
}


fn produce_duty_packet(max_duty: u32, rgb: (u8, u8, u8)) -> [u16; 24] {
    let mut result: [u16; 24] = [0; 24];
    let mut idx = 0;

    let d1 = ((max_duty * 7) / 10) as u16;
    let d0 = ((max_duty * 3) / 10) as u16;

    for &byte in &[rgb.1, rgb.0, rgb.2] {
        for bit in (0..8).rev() {
            let is_one = ((byte >> bit) & 1) != 0;
            result[idx] = if is_one { d1 } else { d0 };
            idx += 1
        }
    }
    info!("Result {}", result);
    result
}

fn produce_duty_packet8(max_duty: u32, rgb: [(u8, u8, u8); 8]) -> [u16; 192] {
    let mut result: [u16; 192] = [0; 192];
    let mut idx = 0;

    let d1 = ((max_duty * 7) / 10) as u16;
    let d0 = ((max_duty * 3) / 10) as u16;

    for col in rgb.iter() {
        for &byte in &[col.1, col.0, col.2] {
            for bit in (0..8).rev() {
                let is_one = ((byte >> bit) & 1) != 0;
                result[idx] = if is_one { d1 } else { d0 };
                idx += 1
            }
        }
    }

    result
}