// Plays an `InputStep` script through real OS input (XTest on Linux/X11).
//
// Letters ride as physical key taps: the X keycode of the US-layout key goes
// through `raw`, with left Shift held for uppercase. `text` is deliberately
// unused: on X11 it binds a spare keycode to the requested keysym, so the
// probe would see a different physical key than the one the layout maps the
// character to (decision D1).

use enigo::{Button, Coordinate, Direction, Enigo, Key, Keyboard, Mouse, Settings};
use freenet_libp2p_bevy_core::testing::NamedKey;
use input_fidelity_e2e::args::PROBE_WINDOW_TITLE;
use input_fidelity_e2e::window_geometry::WindowGeometry;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::{AtomEnum, ConnectionExt, InputFocus};
use x11rb::rust_connection::RustConnection;
use x11rb::wrapper::ConnectionExt as _;

use crate::input_step::InputStep;

// Left Shift on a standard `pc105` keymap (`evdev` 42, X adds 8).
const SHIFT_KEYCODE: u16 = 50;

// X keycode of `character` on a standard US `pc105` keymap (`evdev` + 8).
// Returns `None` for characters the fidelity script never types.
const fn us_keycode(character: char) -> Option<u16> {
    match character {
        'a' | 'A' => Some(38),
        'b' | 'B' => Some(56),
        'c' | 'C' => Some(54),
        'd' | 'D' => Some(40),
        'e' | 'E' => Some(26),
        'f' | 'F' => Some(41),
        'g' | 'G' => Some(42),
        'h' | 'H' => Some(43),
        'i' | 'I' => Some(31),
        'j' | 'J' => Some(44),
        'k' | 'K' => Some(45),
        'l' | 'L' => Some(46),
        'm' | 'M' => Some(58),
        'n' | 'N' => Some(57),
        'o' | 'O' => Some(32),
        'p' | 'P' => Some(33),
        'q' | 'Q' => Some(24),
        'r' | 'R' => Some(27),
        's' | 'S' => Some(39),
        't' | 'T' => Some(28),
        'u' | 'U' => Some(30),
        'v' | 'V' => Some(55),
        'w' | 'W' => Some(25),
        'x' | 'X' => Some(53),
        'y' | 'Y' => Some(29),
        'z' | 'Z' => Some(52),
        '1' => Some(10),
        '2' => Some(11),
        '3' => Some(12),
        '4' => Some(13),
        '5' => Some(14),
        '6' => Some(15),
        '7' => Some(16),
        '8' => Some(17),
        '9' => Some(18),
        '0' => Some(19),
        ' ' => Some(65),
        _ => None,
    }
}

// The named keys' keysyms already live in the default keymap, so `key` reuses
// the existing physical keycode instead of binding a spare one (enigo
// `key_to_keycode` returns early when the keysym is already mapped).
const fn named_key(key: NamedKey) -> Key {
    match key {
        NamedKey::Tab => Key::Tab,
        NamedKey::Enter => Key::Return,
        NamedKey::Backspace => Key::Backspace,
        NamedKey::Escape => Key::Escape,
    }
}

// Rounds a window-local offset to a whole pixel without an `as` cast
// (`as_conversions` is denied): whole script coordinates render as plain
// integers, anything else fails loudly instead of truncating silently.
fn round_to_pixel(offset: f32) -> Result<i32, String> {
    let rounded = offset.round();
    rounded
        .to_string()
        .parse::<i32>()
        .map_err(|_| format!("real driver: offset {rounded} is not a screen pixel"))
}

// Without a window manager X keeps the input focus at `PointerRoot`, and winit
// drops every key event until it has seen a `FocusIn` for its window
// (`x11/event_processor.rs`: `active_window` is only set from `XI_FocusIn`).
// Set the focus explicitly, like clicking the window would under a WM.
fn focus_probe_window(display: &str) -> Result<(), String> {
    let (connection, screen_number) = x11rb::connect(Some(display))
        .map_err(|error| format!("real driver: connect {display} to set input focus: {error}"))?;
    let screen = connection
        .setup()
        .roots
        .get(screen_number)
        .ok_or_else(|| format!("real driver: {display} has no screen {screen_number}"))?;
    let tree = connection
        .query_tree(screen.root)
        .map_err(|error| format!("real driver: query the children of {display} root: {error}"))?
        .reply()
        .map_err(|error| format!("real driver: read the children of {display} root: {error}"))?;
    let probe_window = tree
        .children
        .iter()
        .find(|child| is_probe_window(&connection, **child))
        .copied()
        .ok_or_else(|| format!("real driver: no {PROBE_WINDOW_TITLE:?} window on {display}"))?;
    connection
        .set_input_focus(InputFocus::PARENT, probe_window, x11rb::CURRENT_TIME)
        .map_err(|error| format!("real driver: focus {probe_window} on {display}: {error}"))?;
    connection
        .sync()
        .map_err(|error| format!("real driver: wait for the focus change on {display}: {error}"))
}

fn is_probe_window(connection: &RustConnection, window: u32) -> bool {
    connection
        .get_property(false, window, AtomEnum::WM_NAME, AtomEnum::STRING, 0, 64)
        .ok()
        .and_then(|cookie| cookie.reply().ok())
        .is_some_and(|reply| reply.value == PROBE_WINDOW_TITLE.as_bytes())
}

pub struct RealDriver {
    enigo: Enigo,
    window: WindowGeometry,
}

impl RealDriver {
    /// Connects to the private display only; `DISPLAY` from the environment is
    /// never read. Also moves the X input focus onto the probe window, because
    /// the private Xvfb runs without a window manager.
    ///
    /// # Errors
    /// Returns an error if enigo cannot connect to `display` or the probe
    /// window cannot be found and focused.
    pub fn try_from_geometry(display: &str, window: &WindowGeometry) -> Result<Self, String> {
        let settings = Settings {
            x11_display: Some(display.to_string()),
            ..Settings::default()
        };
        let enigo = Enigo::new(&settings)
            .map_err(|error| format!("real driver: connect to {display}: {error}"))?;
        focus_probe_window(display)?;
        Ok(Self {
            enigo,
            window: *window,
        })
    }

    /// # Errors
    /// Returns an error if any script step cannot be delivered to the display.
    pub fn play(&mut self, script: &[InputStep]) -> Result<(), String> {
        for step in script {
            match step {
                InputStep::ClickAt { x, y } => {
                    let (root_x, root_y) = self.window_to_root(*x, *y)?;
                    self.move_to(root_x, root_y)?;
                    self.enigo
                        .button(Button::Left, Direction::Click)
                        .map_err(|error| {
                            format!("real driver: left click at ({root_x}, {root_y}): {error}")
                        })?;
                }
                InputStep::MoveTo { x, y } => {
                    let (root_x, root_y) = self.window_to_root(*x, *y)?;
                    self.move_to(root_x, root_y)?;
                }
                InputStep::PressKey(key) => {
                    self.enigo
                        .key(named_key(*key), Direction::Click)
                        .map_err(|error| format!("real driver: tap {key:?}: {error}"))?;
                }
                InputStep::TypeText(text) => {
                    for character in text.chars() {
                        self.tap_character(character)?;
                    }
                }
            }
        }
        Ok(())
    }

    fn window_to_root(&self, x: f32, y: f32) -> Result<(i32, i32), String> {
        let root_x = self
            .window
            .x
            .unwrap_or(0)
            .saturating_add(round_to_pixel(x)?);
        let root_y = self
            .window
            .y
            .unwrap_or(0)
            .saturating_add(round_to_pixel(y)?);
        Ok((root_x, root_y))
    }

    fn move_to(&mut self, root_x: i32, root_y: i32) -> Result<(), String> {
        self.enigo
            .move_mouse(root_x, root_y, Coordinate::Abs)
            .map_err(|error| format!("real driver: move to ({root_x}, {root_y}): {error}"))
    }

    fn tap_character(&mut self, character: char) -> Result<(), String> {
        let keycode = us_keycode(character)
            .ok_or_else(|| format!("real driver: no US keycode for {character:?}"))?;
        if character.is_ascii_uppercase() {
            self.enigo
                .raw(SHIFT_KEYCODE, Direction::Press)
                .map_err(|error| format!("real driver: hold Shift for {character:?}: {error}"))?;
        }
        let outcome = self.enigo.raw(keycode, Direction::Click).map_err(|error| {
            format!("real driver: tap {character:?} (keycode {keycode}): {error}")
        });
        if character.is_ascii_uppercase() {
            self.enigo
                .raw(SHIFT_KEYCODE, Direction::Release)
                .map_err(|error| {
                    format!("real driver: release Shift after {character:?}: {error}")
                })?;
        }
        outcome
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use std::time::Duration;

    use freenet_libp2p_bevy_core::testing::harness::E2EDisplay;

    use super::{RealDriver, us_keycode};
    use crate::input_script::INPUT_SCRIPT;
    use crate::probe::ProbePeer;

    const PROBE_BIN: &str = env!("CARGO_BIN_EXE_input_probe");

    #[test]
    fn us_keycodes_cover_the_script_text() {
        for character in "aB1 ".chars() {
            assert!(
                us_keycode(character).is_some(),
                "no keycode for {character:?}"
            );
        }
        assert_eq!(us_keycode('a'), us_keycode('A'));
        assert_eq!(us_keycode('!'), None);
    }

    #[tokio::test(flavor = "multi_thread")]
    #[ignore = "needs a private Xvfb and the lavapipe ICD; run with: just input-fidelity"]
    async fn real_driver_plays_the_script() -> Result<(), String> {
        let run_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(".local-run")
            .join(format!("input-fidelity-real-smoke-{}", std::process::id()));
        std::fs::create_dir_all(&run_dir).map_err(|error| error.to_string())?;
        let display = E2EDisplay::from_env(&run_dir)?
            .ok_or_else(|| "this smoke test only runs on Linux/X11".to_string())?;
        let probe = ProbePeer::try_from_launch(Path::new(PROBE_BIN), &display.name).await?;
        probe.wait_ready().await?;
        let geometry = probe.window().await?;
        let mut driver = RealDriver::try_from_geometry(&display.name, &geometry)?;
        probe.wait_for_focus().await?;
        probe.clear().await?;
        driver.play(INPUT_SCRIPT)?;
        tokio::time::sleep(Duration::from_millis(700)).await;
        let records = probe.records().await?;
        probe.exit().await?;
        if records.is_empty() {
            return Err("the real driver left no records in the probe".to_string());
        }
        Ok(())
    }
}
