//! A4988 Driver
//!
//! Platform-agnostic driver API for the A4988 stepper motor driver. Can be
//! used on any platform for which implementations of the required
//! [embedded-hal] traits are available.
//!
//! For the most part, users are not expected to use this API directly. Please
//! check out [`Stepper`](crate::Stepper) instead.
//!
//! [embedded-hal]: https://crates.io/crates/embedded-hal

use core::convert::Infallible;

use embedded_hal::digital::{OutputPin, PinState};
use fugit::MillisDurationU32 as Milliseconds;
use fugit::NanosDurationU32 as Nanoseconds;

use crate::{
    step_mode::StepMode16,
    traits::{
        EnableDirectionControl, EnableSleepModeControl, EnableStepControl,
        EnableStepModeControl, SetDirection, SetSleepMode, SetStepMode,
        Step as StepTrait,
    },
};

/// The A4988 driver API
///
/// Users are not expected to use this API directly, except to create an
/// instance using [`A4988::new`]. Please check out
/// [`Stepper`](crate::Stepper) instead.
pub struct A4988<Enable, Fault, Sleep, Reset, Mode0, Mode1, Mode2, Step, Dir> {
    enable: Enable,
    fault: Fault,
    sleep: Sleep,
    reset: Reset,
    mode0: Mode0,
    mode1: Mode1,
    mode2: Mode2,
    step: Step,
    dir: Dir,
}

impl A4988<(), (), (), (), (), (), (), (), ()> {
    /// Create a new instance of `A4988`
    pub fn new() -> Self {
        Self {
            enable: (),
            fault: (),
            sleep: (),
            reset: (),
            mode0: (),
            mode1: (),
            mode2: (),
            step: (),
            dir: (),
        }
    }
}

impl<Reset, Mode0, Mode1, Mode2, Step, Dir, OutputPinError>
    EnableStepModeControl<(Reset, Mode0, Mode1, Mode2)>
    for A4988<(), (), (), (), (), (), (), Step, Dir>
where
    Reset: OutputPin<Error = OutputPinError>,
    Mode0: OutputPin<Error = OutputPinError>,
    Mode1: OutputPin<Error = OutputPinError>,
    Mode2: OutputPin<Error = OutputPinError>,
{
    type WithStepModeControl =
        A4988<(), (), (), Reset, Mode0, Mode1, Mode2, Step, Dir>;

    fn enable_step_mode_control(
        self,
        (reset, mode0, mode1, mode2): (Reset, Mode0, Mode1, Mode2),
    ) -> Self::WithStepModeControl {
        A4988 {
            enable: self.enable,
            fault: self.fault,
            sleep: self.sleep,
            reset,
            mode0,
            mode1,
            mode2,
            step: self.step,
            dir: self.dir,
        }
    }
}

impl<Reset, Mode0, Mode1, Mode2, Step, Dir, OutputPinError> SetStepMode
    for A4988<(), (), (), Reset, Mode0, Mode1, Mode2, Step, Dir>
where
    Reset: OutputPin<Error = OutputPinError>,
    Mode0: OutputPin<Error = OutputPinError>,
    Mode1: OutputPin<Error = OutputPinError>,
    Mode2: OutputPin<Error = OutputPinError>,
{
    // Timing Requirements (page 6)
    // https://www.pololu.com/file/0J450/A4988.pdf
    const SETUP_TIME: Nanoseconds = Nanoseconds::from_ticks(200);
    const HOLD_TIME: Nanoseconds = Nanoseconds::from_ticks(200);

    type Error = OutputPinError;
    type StepMode = StepMode16;

    fn apply_mode_config(
        &mut self,
        step_mode: Self::StepMode,
    ) -> Result<(), Self::Error> {
        use PinState::*;
        use StepMode16::*;
        let (mode0, mode1, mode2) = match step_mode {
            Full => (Low, Low, Low),
            M2 => (High, Low, Low),
            M4 => (Low, High, Low),
            M8 => (High, High, Low),
            M16 => (High, High, High),
        };

        // Set mode signals.
        self.mode0.set_state(mode0)?;
        self.mode1.set_state(mode1)?;
        self.mode2.set_state(mode2)?;

        Ok(())
    }

    fn enable_driver(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl<Reset, Mode0, Mode1, Mode2, Step, Dir, OutputPinError>
    EnableDirectionControl<Dir>
    for A4988<(), (), (), Reset, Mode0, Mode1, Mode2, Step, ()>
where
    Dir: OutputPin<Error = OutputPinError>,
{
    type WithDirectionControl =
        A4988<(), (), (), Reset, Mode0, Mode1, Mode2, Step, Dir>;

    fn enable_direction_control(self, dir: Dir) -> Self::WithDirectionControl {
        A4988 {
            enable: self.enable,
            fault: self.fault,
            sleep: self.sleep,
            reset: self.reset,
            mode0: self.mode0,
            mode1: self.mode1,
            mode2: self.mode2,
            step: self.step,
            dir,
        }
    }
}

impl<Reset, Mode0, Mode1, Mode2, Step, Dir, OutputPinError> SetDirection
    for A4988<(), (), (), Reset, Mode0, Mode1, Mode2, Step, Dir>
where
    Dir: OutputPin<Error = OutputPinError>,
{
    // Timing Requirements (page 6)
    // https://www.pololu.com/file/0J450/A4988.pdf
    const SETUP_TIME: Nanoseconds = Nanoseconds::from_ticks(200);

    type Dir = Dir;
    type Error = Infallible;

    fn dir(&mut self) -> Result<&mut Self::Dir, Self::Error> {
        Ok(&mut self.dir)
    }
}

impl<Reset, Mode0, Mode1, Mode2, Step, Dir, OutputPinError>
    EnableStepControl<Step>
    for A4988<(), (), (), Reset, Mode0, Mode1, Mode2, (), Dir>
where
    Step: OutputPin<Error = OutputPinError>,
{
    type WithStepControl =
        A4988<(), (), (), Reset, Mode0, Mode1, Mode2, Step, Dir>;

    fn enable_step_control(self, step: Step) -> Self::WithStepControl {
        A4988 {
            enable: self.enable,
            fault: self.fault,
            sleep: self.sleep,
            reset: self.reset,
            mode0: self.mode0,
            mode1: self.mode1,
            mode2: self.mode2,
            step,
            dir: self.dir,
        }
    }
}

impl<Reset, Mode0, Mode1, Mode2, Step, Dir, OutputPinError> StepTrait
    for A4988<(), (), (), Reset, Mode0, Mode1, Mode2, Step, Dir>
where
    Step: OutputPin<Error = OutputPinError>,
{
    // Timing Requirements (page 6)
    // https://www.pololu.com/file/0J450/A4988.pdf
    const PULSE_LENGTH: Nanoseconds = Nanoseconds::from_ticks(1000); // 1µs

    type Step = Step;
    type Error = Infallible;

    fn step(&mut self) -> Result<&mut Self::Step, Self::Error> {
        Ok(&mut self.step)
    }
}

impl<Sleep, OutputPinError> EnableSleepModeControl<Sleep>
    for A4988<(), (), Sleep, (), (), (), (), (), ()>
where
    Sleep: OutputPin<Error = OutputPinError>,
{
    type WithSleepModeControl = A4988<(), (), Sleep, (), (), (), (), (), ()>;

    fn enable_sleep_mode_control(
        self,
        sleep: Sleep,
    ) -> Self::WithSleepModeControl {
        A4988 {
            enable: self.enable,
            fault: self.fault,
            sleep,
            reset: self.reset,
            mode0: self.mode0,
            mode1: self.mode1,
            mode2: self.mode2,
            step: self.step,
            dir: self.dir,
        }
    }
}

impl<Sleep, OutputPinError> SetSleepMode
    for A4988<(), (), Sleep, (), (), (), (), (), ()>
where
    Sleep: OutputPin<Error = OutputPinError>,
{
    // Timing Requirements (page 6)
    // https://www.pololu.com/file/0J450/A4988.pdf
    const SETUP_TIME: Nanoseconds = Nanoseconds::from_ticks(200);

    // Sleep mode (page 10)
    // https://www.pololu.com/file/0J450/A4988.pdf
    const WAKE_UP_TIME: Nanoseconds = Milliseconds::from_ticks(1).convert();

    type Sleep = Sleep;
    type Error = Infallible;

    fn sleep(&mut self) -> Result<&mut Self::Sleep, Self::Error> {
        Ok(&mut self.sleep)
    }
}
