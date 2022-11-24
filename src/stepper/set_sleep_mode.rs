use crate::tracing::*;
use crate::traits::SetSleepMode;
use core::task::Poll;

use embedded_hal::digital::ErrorType;
use embedded_hal::digital::OutputPin;
use fugit::TimerDurationU32 as TimerDuration;
use fugit_timer::Timer as TimerTrait;

use super::SignalError;

/// The "future" returned by [`Stepper::sleep`] and [`Stepper::wakeup`]
///
/// Please note that this type provides a custom API and does not implement
/// [`core::future::Future`]. This might change, when using futures for embedded
/// development becomes more practical.
#[must_use]
pub struct SetSleepModeFuture<Driver, Timer, const TIMER_HZ: u32> {
    sleep_mode_enabled: bool,
    driver: Driver,
    timer: Timer,
    state: State,
}

impl<Driver, Timer, const TIMER_HZ: u32>
    SetSleepModeFuture<Driver, Timer, TIMER_HZ>
where
    Driver: SetSleepMode,
    Timer: TimerTrait<TIMER_HZ>,
{
    /// Create new instance of `SetSleepModeFuture`
    ///
    /// This constructor is public to provide maximum flexibility for non-standard use
    /// cases. Most users can ignore this and just use [`Stepper::sleep`] and
    /// [`Stepper::wakeup`] instead.
    pub fn new(sleep_mode_enabled: bool, driver: Driver, timer: Timer) -> Self {
        Self {
            sleep_mode_enabled,
            driver,
            timer,
            state: State::Initial,
        }
    }

    /// Poll the future
    ///
    /// The future must be polled for the operation to make progress. The
    /// operation won't start, until this method has been called once. Returns
    /// [`Poll::Pending`], if the operation is not finished yet, or
    /// [`Poll::Ready`], once it is.
    ///
    /// If this method returns [`Poll::Pending`], the user can opt to keep
    /// calling it at a high frequency (see [`Self::wait`]) until the operation
    /// completes, or set up an interrupt that fires once the timer finishes
    /// counting down, and call this method again once it does.
    pub fn poll(
        &mut self,
    ) -> Poll<
        Result<
            (),
            SignalError<
                Driver::Error,
                <Driver::Sleep as ErrorType>::Error,
                Timer::Error,
            >,
        >,
    > {
        match self.state {
            State::Initial => {
                trace!("setting sleep mode to {}", self.sleep_mode_enabled);
                if self.sleep_mode_enabled {
                    self.driver
                        .sleep()
                        .map_err(|err| SignalError::PinUnavailable(err))?
                        .set_low()
                        .map_err(|err| SignalError::Pin(err))?;
                    self.state = State::SleepModeSet;
                } else {
                    self.driver
                        .sleep()
                        .map_err(|err| SignalError::PinUnavailable(err))?
                        .set_high()
                        .map_err(|err| SignalError::Pin(err))?;
                    self.state = State::WakingUp;
                }

                let ticks: TimerDuration<TIMER_HZ> =
                    Driver::SETUP_TIME.convert();
                self.timer
                    .start(ticks)
                    .map_err(|err| SignalError::Timer(err))?;
                trace!(
                    "waiting for setup to complete ({}ns)",
                    ticks.to_nanos()
                );

                Poll::Pending
            }
            State::WakingUp => match self.timer.wait() {
                Ok(()) => {
                    let ticks: TimerDuration<TIMER_HZ> =
                        Driver::WAKE_UP_TIME.convert();
                    self.timer
                        .start(ticks)
                        .map_err(|err| SignalError::Timer(err))?;
                    trace!(
                        "waiting for driver to wakeup ({}ns)",
                        ticks.to_nanos()
                    );

                    self.state = State::SleepModeSet;
                    Poll::Ready(Ok(()))
                }
                Err(nb::Error::Other(err)) => {
                    self.state = State::Finished;
                    Poll::Ready(Err(SignalError::Timer(err)))
                }
                Err(nb::Error::WouldBlock) => Poll::Pending,
            },
            State::SleepModeSet => match self.timer.wait() {
                Ok(()) => {
                    self.state = State::Finished;
                    Poll::Ready(Ok(()))
                }
                Err(nb::Error::Other(err)) => {
                    self.state = State::Finished;
                    Poll::Ready(Err(SignalError::Timer(err)))
                }
                Err(nb::Error::WouldBlock) => Poll::Pending,
            },
            State::Finished => Poll::Ready(Ok(())),
        }
    }

    /// Wait until the operation completes
    ///
    /// This method will call [`Self::poll`] in a busy loop until the operation
    /// has finished.
    pub fn wait(
        &mut self,
    ) -> Result<
        (),
        SignalError<
            Driver::Error,
            <Driver::Sleep as ErrorType>::Error,
            Timer::Error,
        >,
    > {
        loop {
            if let Poll::Ready(result) = self.poll() {
                return result;
            }
        }
    }

    /// Drop the future and release the resources that were moved into it
    pub fn release(self) -> (Driver, Timer) {
        (self.driver, self.timer)
    }
}

enum State {
    Initial,
    WakingUp,
    SleepModeSet,
    Finished,
}
