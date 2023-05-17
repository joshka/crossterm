use std::time::Duration;

use crossterm_winapi::{Console, Handle, InputRecord};

use crate::event::{
    sys::windows::{
        parse::{surrogate_pair_to_key_event, MouseButtonsPressed, WindowsKeyEvent},
        poll::WinApiPoll,
        surrogate::{HighSurrogate, LowSurrogate},
    },
    Event, KeyEventKind,
};

#[cfg(feature = "event-stream")]
use crate::event::sys::Waker;
use crate::event::{
    source::EventSource,
    sys::windows::parse::{handle_mouse_event, parse_key_event_record},
    timeout::PollTimeout,
    InternalEvent,
};

pub(crate) struct WindowsEventSource {
    console: Console,
    poll: WinApiPoll,
    pressed_high_surrogate: Option<HighSurrogate>,
    released_high_surrogate: Option<HighSurrogate>,
    repeated_high_surrogate: Option<HighSurrogate>,
    mouse_buttons_pressed: MouseButtonsPressed,
}

impl WindowsEventSource {
    pub(crate) fn new() -> std::io::Result<WindowsEventSource> {
        let console = Console::from(Handle::current_in_handle()?);
        Ok(WindowsEventSource {
            console,

            #[cfg(not(feature = "event-stream"))]
            poll: WinApiPoll::new(),
            #[cfg(feature = "event-stream")]
            poll: WinApiPoll::new()?,

            pressed_high_surrogate: None,
            released_high_surrogate: None,
            repeated_high_surrogate: None,
            mouse_buttons_pressed: MouseButtonsPressed::default(),
        })
    }
}

impl EventSource for WindowsEventSource {
    fn try_read(&mut self, timeout: Option<Duration>) -> std::io::Result<Option<InternalEvent>> {
        let poll_timeout = PollTimeout::new(timeout);

        loop {
            if let Some(event_ready) = self.poll.poll(poll_timeout.leftover())? {
                let number = self.console.number_of_console_input_events()?;
                if event_ready && number != 0 {
                    let event = match self.console.read_single_input_event()? {
                        InputRecord::KeyEvent(record) => match parse_key_event_record(&record) {
                            Some(WindowsKeyEvent::KeyEvent(key_event)) => {
                                Some(Event::Key(key_event))
                            }
                            Some(WindowsKeyEvent::HighSurrogate(HighSurrogate {
                                high,
                                modifiers,
                                key_event_kind: key_event_kind @ KeyEventKind::Press,
                                key_event_state,
                            })) => {
                                if let Some(HighSurrogate {
                                    high: pending_high, ..
                                }) = self.pressed_high_surrogate.replace(HighSurrogate {
                                    high,
                                    modifiers,
                                    key_event_kind,
                                    key_event_state,
                                }) {
                                    panic!("two high surrogates {pending_high:?} and {high:?}");
                                } else {
                                    continue;
                                }
                            }
                            Some(WindowsKeyEvent::HighSurrogate(HighSurrogate {
                                high,
                                modifiers,
                                key_event_kind: key_event_kind @ KeyEventKind::Release,
                                key_event_state,
                            })) => {
                                if let Some(HighSurrogate {
                                    high: pending_high, ..
                                }) = self.released_high_surrogate.replace(HighSurrogate {
                                    high,
                                    modifiers,
                                    key_event_kind,
                                    key_event_state,
                                }) {
                                    panic!("two high surrogates {pending_high:?} and {high:?}");
                                } else {
                                    continue;
                                }
                            }
                            Some(WindowsKeyEvent::HighSurrogate(HighSurrogate {
                                high,
                                modifiers,
                                key_event_kind: key_event_kind @ KeyEventKind::Repeat,
                                key_event_state,
                            })) => {
                                if let Some(HighSurrogate {
                                    high: pending_high, ..
                                }) = self.repeated_high_surrogate.replace(HighSurrogate {
                                    high,
                                    modifiers,
                                    key_event_kind,
                                    key_event_state,
                                }) {
                                    panic!("two high surrogates {pending_high:?} and {high:?}");
                                } else {
                                    continue;
                                }
                            }
                            Some(WindowsKeyEvent::LowSurrogate(LowSurrogate {
                                low,
                                key_event_kind: KeyEventKind::Press,
                                ..
                            })) => match self.pressed_high_surrogate.take() {
                                Some(HighSurrogate {
                                    high,
                                    modifiers,
                                    key_event_kind,
                                    key_event_state,
                                }) => Some(surrogate_pair_to_key_event(
                                    high,
                                    low,
                                    modifiers,
                                    key_event_kind,
                                    key_event_state,
                                )),
                                None => panic!("missing high surrogate {low:?}"),
                            },
                            Some(WindowsKeyEvent::LowSurrogate(LowSurrogate {
                                low,
                                key_event_kind: KeyEventKind::Release,
                                ..
                            })) => match self.released_high_surrogate.take() {
                                Some(HighSurrogate {
                                    high,
                                    modifiers,
                                    key_event_kind,
                                    key_event_state,
                                }) => Some(surrogate_pair_to_key_event(
                                    high,
                                    low,
                                    modifiers,
                                    key_event_kind,
                                    key_event_state,
                                )),
                                None => panic!("missing high surrogate {low:?}"),
                            },
                            Some(WindowsKeyEvent::LowSurrogate(LowSurrogate {
                                low,
                                key_event_kind: KeyEventKind::Repeat,
                                ..
                            })) => match self.repeated_high_surrogate.take() {
                                Some(HighSurrogate {
                                    high,
                                    modifiers,
                                    key_event_kind,
                                    key_event_state,
                                }) => Some(surrogate_pair_to_key_event(
                                    high,
                                    low,
                                    modifiers,
                                    key_event_kind,
                                    key_event_state,
                                )),
                                None => panic!("missing high surrogate {low:?}"),
                            },
                            None => None,
                        },
                        InputRecord::MouseEvent(record) => {
                            let mouse_event =
                                handle_mouse_event(record, &self.mouse_buttons_pressed);
                            self.mouse_buttons_pressed = MouseButtonsPressed {
                                left: record.button_state.left_button(),
                                right: record.button_state.right_button(),
                                middle: record.button_state.middle_button(),
                            };

                            mouse_event
                        }
                        InputRecord::WindowBufferSizeEvent(record) => {
                            // windows starts counting at 0, unix at 1, add one to replicate unix behaviour.
                            Some(Event::Resize(
                                record.size.x as u16 + 1,
                                record.size.y as u16 + 1,
                            ))
                        }
                        InputRecord::FocusEvent(record) => {
                            let event = if record.set_focus {
                                Event::FocusGained
                            } else {
                                Event::FocusLost
                            };
                            Some(event)
                        }
                        _ => None,
                    };

                    if let Some(event) = event {
                        return Ok(Some(InternalEvent::Event(event)));
                    }
                }
            }

            if poll_timeout.elapsed() {
                return Ok(None);
            }
        }
    }

    #[cfg(feature = "event-stream")]
    fn waker(&self) -> Waker {
        self.poll.waker()
    }
}
