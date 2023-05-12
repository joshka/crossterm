//! Demonstrates how to block read characters or a full line.
//! Just note that crossterm is not required to do this and can be done with `io::stdin()`.
//!
//! cargo run --example event-read-char-line

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Read line until enter key is hit. Press Ctrl+C to exit.");

    let mut pressed_code_points = String::new();
    let mut released_code_points = String::new();
    let mut repeated_code_points = String::new();
    loop {
        match event::read()? {
            Event::Key(KeyEvent {
                code: KeyCode::Char(ch),
                modifiers,
                kind: KeyEventKind::Press,
                state,
            }) => {
                pressed_code_points.push(ch);
                println!("code: KeyCode::Char({ch:?}), modifiers: {modifiers:?}, kind: KeyEventKind::Press, state: {state:?}");
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char(ch),
                modifiers,
                kind: KeyEventKind::Release,
                state,
            }) => {
                released_code_points.push(ch);
                println!("code: KeyCode::Char({ch:?}), modifiers: {modifiers:?}, kind: KeyEventKind::Release, state: {state:?}");
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char(ch),
                modifiers,
                kind: KeyEventKind::Repeat,
                state,
            }) => {
                repeated_code_points.push(ch);
                println!("code: KeyCode::Char({ch:?}), modifiers: {modifiers:?}, kind: KeyEventKind::Repeat, state: {state:?}");
            }
            Event::Key(KeyEvent {
                code: KeyCode::Enter,
                modifiers,
                kind: KeyEventKind::Press,
                state,
            }) if !pressed_code_points.is_empty() => {
                dbg!(&pressed_code_points);
                println!("abstract_characters: {pressed_code_points}, modifiers: {modifiers:?}, kind: KeyEventKind::Press, state: {state:?}");
                pressed_code_points.clear();
            }
            Event::Key(KeyEvent {
                code: KeyCode::Enter,
                modifiers,
                kind: KeyEventKind::Press,
                state,
            }) => {
                println!("code: KeyCode::Enter, modifiers: {modifiers:?}, kind: KeyEventKind::Press, state: {state:?}");
            }
            Event::Key(KeyEvent {
                code: KeyCode::Enter,
                modifiers,
                kind: KeyEventKind::Release,
                state,
            }) if !released_code_points.is_empty() => {
                dbg!(&released_code_points);
                println!("abstract_characters: {released_code_points}, modifiers: {modifiers:?}, kind: KeyEventKind::Release, state: {state:?}");
                released_code_points.clear();
            }
            Event::Key(KeyEvent {
                code: KeyCode::Enter,
                modifiers,
                kind: KeyEventKind::Release,
                state,
            }) => {
                println!("code: KeyCode::Enter, modifiers: {modifiers:?}, kind: KeyEventKind::Release, state: {state:?}");
            }
            Event::Key(KeyEvent {
                code: KeyCode::Enter,
                modifiers,
                kind: KeyEventKind::Repeat,
                state,
            }) if !repeated_code_points.is_empty() => {
                dbg!(&repeated_code_points);
                println!("abstract_characters: {repeated_code_points}, modifiers: {modifiers:?}, kind: KeyEventKind::Repeat, state: {state:?}");
                repeated_code_points.clear();
            }
            Event::Key(KeyEvent {
                code: KeyCode::Enter,
                modifiers,
                kind: KeyEventKind::Repeat,
                state,
            }) => {
                println!("code: KeyCode::Enter, modifiers: {modifiers:?}, kind: KeyEventKind::Repeat, state: {state:?}");
            }
            e @ _ => println!("{e:?}"),
        }
    }
}
