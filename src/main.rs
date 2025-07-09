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

const CHORUS_STAYIN_ALIVE: &[(&str, u32)] = &[
    // A A A A
    
    // stayin
    ("F#4", 500), ("_", 250), // F#4 + пауза восьмая
    ("G#4", 250),             // «‑in»
    // alive
    ("A4", 750), ("_", 250),             // протяжно «a‑live»

    // stayin alive (повтор)
    ("F#4", 500), ("_", 250),
    ("G#4", 250),
    ("A4", 750), ("_", 250),

    // четверка A A A A
    ("A4", 250), ("_", 250), ("A4", 250), ("_", 250), ("A4", 250), ("_", 250), ("A4", 250), ("_", 250),
    // stayin
    ("F#4", 500), ("_", 250),
    ("G#4", 250),
    // alive
    ("A4", 750), ("_", 250),

    // еще раз A A A A
    ("A4", 250), ("_", 250), ("A4", 250), ("_", 250), ("A4", 250), ("_", 250), ("A4", 250), ("_", 250),
    // stayin
    ("F#4", 500), ("_", 250),
    ("G#4", 250),
    // alive
    ("A4", 750), ("_", 250),

    // финальный «stayin alive» с протяжной модуляцией
    ("F#4", 500),              // «stay-»
    ("G#4", 500),              // «‑in»
    ("A4", 1000),              // «a‑live» — протяжно 2 четверти
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