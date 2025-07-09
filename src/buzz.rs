use cortex_m::prelude::_embedded_hal_Pwm;
use embassy_stm32::gpio::OutputType;
use embassy_stm32::Peri;
use embassy_stm32::time::{hz, Hertz};
use embassy_stm32::timer::{Ch1, Ch2, Ch3, Ch4, Channel, GeneralInstance4Channel, TimerPin, UpDma};
use embassy_stm32::timer::low_level::CountingMode;
use embassy_stm32::timer::simple_pwm::{PwmPin, SimplePwm};
use embassy_time::{Duration, Timer};
use crate::fmt::info;

pub struct Buzzer<'a, T: GeneralInstance4Channel> {
    notes: Option<[(Hertz, u32); 256]>,
    pwm: SimplePwm<'a, T>,
    channel: Channel,
}

impl<'a, T> Buzzer<'a, T>
where
    T: GeneralInstance4Channel + 'a,
{
    pub fn new_ch1(
        timer: Peri<'a, T>,
        pin: Peri<'a, impl TimerPin<T, Ch1>>,
    ) -> Self {
        info!("Creating buzzer on channel 1");
        let pwm_pin = PwmPin::new(pin, OutputType::PushPull);
        let pwm_driver = SimplePwm::new(
            timer, Some(pwm_pin), None, None, None, hz(2000), CountingMode::EdgeAlignedUp
        );
        Self {notes: None, pwm: pwm_driver, channel:Channel::Ch1}

    }
    pub fn new_ch2(
        timer: Peri<'a, T>,
        pin: Peri<'a, impl TimerPin<T, Ch2>>,
    ) -> Self {
        info!("Creating buzzer on channel 2");
        let pwm_pin = PwmPin::new(pin, OutputType::PushPull);
        let pwm_driver = SimplePwm::new(
            timer, None, Some(pwm_pin), None, None, hz(2000), CountingMode::EdgeAlignedUp
        );
        Self {notes: None, pwm: pwm_driver, channel:Channel::Ch2}
    }

    pub fn new_ch3(
        timer: Peri<'a, T>,
        pin: Peri<'a, impl TimerPin<T, Ch3>>,
    ) -> Self {
        info!("Creating buzzer on channel 3");
        let pwm_pin = PwmPin::new(pin, OutputType::PushPull);
        let pwm_driver = SimplePwm::new(
            timer, None, None, Some(pwm_pin), None, hz(2000), CountingMode::EdgeAlignedUp
        );
        Self {notes: None, pwm: pwm_driver, channel:Channel::Ch3}
    }

    pub fn new_ch4(
        timer: Peri<'a, T>,
        pin: Peri<'a, impl TimerPin<T, Ch4>>,
    ) -> Self {
        info!("Creating buzzer on channel 4");
        let pwm_pin = PwmPin::new(pin, OutputType::PushPull);
        let pwm_driver = SimplePwm::new(
            timer, None, None, None, Some(pwm_pin), hz(2000), CountingMode::EdgeAlignedUp
        );
        Self {notes: None, pwm: pwm_driver, channel:Channel::Ch4}
    }
}





impl<'a, T> Buzzer<'a, T>
where
    T: GeneralInstance4Channel + 'a,
{
    pub fn set_notes(&mut self, notes: &'a [(&str, u32)]) {
        let mut freqs = [(Hertz(0), 0); 256];
        let mut idx = 0;
        for &(note, dur) in notes {
            let Some(freq) = note_to_freq(note) else {
                info!("Could resolve notes: {:?}", note);
                break;
            };
            freqs[idx] = (freq, dur);
            idx += 1;
        }
        self.notes = Some(freqs);
    }

    pub async fn buzzer_task(&mut self) {
        for note in self.notes.as_ref().unwrap().iter() {
            if note.0 != hz(0) {
                self.pwm.set_frequency(note.0);
                self.pwm.set_duty(self.channel, self.pwm.get_max_duty() / 2);   
                self.pwm.enable(self.channel);
                Timer::after_millis(note.1 as u64).await;
                self.pwm.disable(self.channel);
            } else {
                Timer::after_millis(note.1 as u64).await;
            }
        }
    }
}

fn note_to_freq(note: &str) -> Option<Hertz> {
    let raw = match note {
        "C4"  => 261,
        "C#4" => 277,
        "D4"  => 294,
        "D#4" => 311,
        "E4"  => 329,
        "F4"  => 349,
        "F#4" => 370,
        "G4"  => 392,
        "G#4" => 415,
        "A4"  => 440,
        "A#4" => 466,
        "B4"  => 493,
        "C5"  => 523,
        "C#5" => 554,
        "D5"  => 587,
        "D#5" => 622,
        "E5"  => 659,
        "F5"  => 698,
        "F#5" => 740,
        "G5"  => 784,
        "G#5" => 830,
        "A5"  => 880,
        "A#5" => 932,
        "B5"  => 987,
        "C6"  => 1046,
        "_"   => 0,
         _    => -1,
    };
    if raw >= 0 {
        Some(hz(raw as u32))
    } else {
        None
    }
}

#[macro_export]
macro_rules! buzzer {
    ($pin:expr, $channel:expr, $timer:expr) => {
        match $channel {
            Ch1 => crate::buzzer::Buzzer::new_ch1($timer, $pin),
            Ch2 => crate::buzzer::Buzzer::new_ch2($timer, $pin),
            Ch3 => crate::buzzer::Buzzer::new_ch3($timer, $pin),
            Ch4 => crate::buzzer::Buzzer::new_ch4($timer, $pin),
        }
    }
}

