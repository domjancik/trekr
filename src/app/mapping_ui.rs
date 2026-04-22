use crate::actions::AppAction;
use sdl3::event::Event;
use sdl3::keyboard::{Keycode, Mod};
use sdl3::rect::Rect;

use super::types::DiscoverabilityTarget;

pub(super) fn mapping_target_label_for_action(action: AppAction) -> Option<&'static str> {
    match action {
        AppAction::TogglePlayback => Some("Play/Stop"),
        AppAction::ToggleRecording => Some("Record"),
        AppAction::CycleRecordMode => Some("Record Mode"),
        AppAction::ToggleLoopRecordingExtension => Some("Loop Recording Wrap"),
        AppAction::ToggleGlobalLoop => Some("Song Loop"),
        AppAction::CycleGlobalHarmonyRoot => Some("Global Harmony Root"),
        AppAction::ResetGlobalLoop => Some("Reset Song Loop"),
        AppAction::ToggleCurrentTrackLoop => Some("Track Loop"),
        AppAction::ToggleStoredLoopRecallQuantize => Some("Stored Loop Recall Quantize"),
        AppAction::CycleStoredLoopLaunchQuantize => Some("Stored Loop Launch Quantize"),
        AppAction::RecallStoredLoopSlot1 => Some("Recall Stored Loop Slot 1"),
        AppAction::RecallStoredLoopSlot2 => Some("Recall Stored Loop Slot 2"),
        AppAction::RecallStoredLoopSlot3 => Some("Recall Stored Loop Slot 3"),
        AppAction::RecallStoredLoopSlot4 => Some("Recall Stored Loop Slot 4"),
        AppAction::RecallStoredLoopSlot5 => Some("Recall Stored Loop Slot 5"),
        AppAction::RecallStoredLoopSlot6 => Some("Recall Stored Loop Slot 6"),
        AppAction::RecallStoredLoopSlot7 => Some("Recall Stored Loop Slot 7"),
        AppAction::RecallStoredLoopSlot8 => Some("Recall Stored Loop Slot 8"),
        AppAction::StoreCurrentLoopToSlot1 => Some("Store Current Loop To Slot 1"),
        AppAction::StoreCurrentLoopToSlot2 => Some("Store Current Loop To Slot 2"),
        AppAction::StoreCurrentLoopToSlot3 => Some("Store Current Loop To Slot 3"),
        AppAction::StoreCurrentLoopToSlot4 => Some("Store Current Loop To Slot 4"),
        AppAction::StoreCurrentLoopToSlot5 => Some("Store Current Loop To Slot 5"),
        AppAction::StoreCurrentLoopToSlot6 => Some("Store Current Loop To Slot 6"),
        AppAction::StoreCurrentLoopToSlot7 => Some("Store Current Loop To Slot 7"),
        AppAction::StoreCurrentLoopToSlot8 => Some("Store Current Loop To Slot 8"),
        AppAction::ClearStoredLoopSlot1 => Some("Clear Stored Loop Slot 1"),
        AppAction::ClearStoredLoopSlot2 => Some("Clear Stored Loop Slot 2"),
        AppAction::ClearStoredLoopSlot3 => Some("Clear Stored Loop Slot 3"),
        AppAction::ClearStoredLoopSlot4 => Some("Clear Stored Loop Slot 4"),
        AppAction::ClearStoredLoopSlot5 => Some("Clear Stored Loop Slot 5"),
        AppAction::ClearStoredLoopSlot6 => Some("Clear Stored Loop Slot 6"),
        AppAction::ClearStoredLoopSlot7 => Some("Clear Stored Loop Slot 7"),
        AppAction::ClearStoredLoopSlot8 => Some("Clear Stored Loop Slot 8"),
        AppAction::ToggleCurrentTrackArm => Some("Track Arm"),
        AppAction::ToggleCurrentTrackMute => Some("Track Mute"),
        AppAction::ToggleCurrentTrackSolo => Some("Track Solo"),
        AppAction::ToggleCurrentTrackPassthrough => Some("Passthrough"),
        AppAction::ToggleCurrentTrackRecordingView => Some("Recording View"),
        AppAction::SelectPreviousRecordingClip => Some("Select Previous Recording Clip"),
        AppAction::SelectNextRecordingClip => Some("Select Next Recording Clip"),
        AppAction::ToggleSelectedRecordingClipMute => Some("Recording Clip Mute"),
        AppAction::DeleteSelectedRecordingClip => Some("Delete Recording Clip"),
        AppAction::ToggleFocusedTrackView => Some("Focused Track View"),
        AppAction::ToggleLinkEnabled => Some("Link Enable"),
        AppAction::ToggleLinkStartStopSync => Some("Link Start/Stop"),
        _ => None,
    }
}

pub(super) fn direct_mapping_key_label(event: &Event) -> Option<String> {
    let Event::KeyDown {
        keycode: Some(keycode),
        keymod,
        repeat: false,
        ..
    } = event
    else {
        return None;
    };

    if matches!(
        keycode,
        Keycode::LShift
            | Keycode::RShift
            | Keycode::LCtrl
            | Keycode::RCtrl
            | Keycode::LAlt
            | Keycode::RAlt
            | Keycode::LGui
            | Keycode::RGui
            | Keycode::Mode
            | Keycode::Escape
            | Keycode::F8
    ) {
        return None;
    }

    let key_label = keycode_mapping_label(*keycode)?;
    Some(with_modifier_prefixes(key_label, *keymod))
}

pub(super) fn mapping_target_lookup_input(event: &Event) -> Option<String> {
    let Event::KeyDown {
        keycode: Some(keycode),
        keymod,
        repeat: false,
        ..
    } = event
    else {
        return None;
    };

    if keymod.intersects(
        Mod::LCTRLMOD | Mod::RCTRLMOD | Mod::LALTMOD | Mod::RALTMOD | Mod::LGUIMOD | Mod::RGUIMOD,
    ) {
        return None;
    }

    let input = match keycode {
        Keycode::Space => " ".to_string(),
        Keycode::Minus => "-".to_string(),
        Keycode::Slash => "/".to_string(),
        Keycode::_0 => "0".to_string(),
        Keycode::_1 => "1".to_string(),
        Keycode::_2 => "2".to_string(),
        Keycode::_3 => "3".to_string(),
        Keycode::_4 => "4".to_string(),
        Keycode::_5 => "5".to_string(),
        Keycode::_6 => "6".to_string(),
        Keycode::_7 => "7".to_string(),
        Keycode::_8 => "8".to_string(),
        Keycode::_9 => "9".to_string(),
        Keycode::A => "a".to_string(),
        Keycode::B => "b".to_string(),
        Keycode::C => "c".to_string(),
        Keycode::D => "d".to_string(),
        Keycode::E => "e".to_string(),
        Keycode::F => "f".to_string(),
        Keycode::G => "g".to_string(),
        Keycode::H => "h".to_string(),
        Keycode::I => "i".to_string(),
        Keycode::J => "j".to_string(),
        Keycode::K => "k".to_string(),
        Keycode::L => "l".to_string(),
        Keycode::M => "m".to_string(),
        Keycode::N => "n".to_string(),
        Keycode::O => "o".to_string(),
        Keycode::P => "p".to_string(),
        Keycode::Q => "q".to_string(),
        Keycode::R => "r".to_string(),
        Keycode::S => "s".to_string(),
        Keycode::T => "t".to_string(),
        Keycode::U => "u".to_string(),
        Keycode::V => "v".to_string(),
        Keycode::W => "w".to_string(),
        Keycode::X => "x".to_string(),
        Keycode::Y => "y".to_string(),
        Keycode::Z => "z".to_string(),
        _ => return None,
    };

    Some(input)
}

fn with_modifier_prefixes(key_label: &str, keymod: Mod) -> String {
    let mut label = String::new();
    if keymod.intersects(Mod::LCTRLMOD | Mod::RCTRLMOD) {
        label.push_str("Ctrl+");
    }
    if keymod.intersects(Mod::LALTMOD | Mod::RALTMOD) {
        label.push_str("Alt+");
    }
    if keymod.intersects(Mod::LSHIFTMOD | Mod::RSHIFTMOD) {
        label.push_str("Shift+");
    }
    label.push_str(key_label);
    label
}

fn keycode_mapping_label(keycode: Keycode) -> Option<&'static str> {
    match keycode {
        Keycode::Space => Some("Space"),
        Keycode::Tab => Some("Tab"),
        Keycode::Return => Some("Enter"),
        Keycode::Delete => Some("Delete"),
        Keycode::Backspace => Some("Backspace"),
        Keycode::Home => Some("Home"),
        Keycode::Left => Some("Left"),
        Keycode::Right => Some("Right"),
        Keycode::Up => Some("Up"),
        Keycode::Down => Some("Down"),
        Keycode::LeftBracket => Some("["),
        Keycode::RightBracket => Some("]"),
        Keycode::Comma => Some(","),
        Keycode::Period => Some("."),
        Keycode::Minus => Some("-"),
        Keycode::Equals => Some("="),
        Keycode::Slash => Some("/"),
        Keycode::Backslash => Some("\\"),
        Keycode::F1 => Some("F1"),
        Keycode::F2 => Some("F2"),
        Keycode::F3 => Some("F3"),
        Keycode::F4 => Some("F4"),
        Keycode::F5 => Some("F5"),
        Keycode::F6 => Some("F6"),
        Keycode::_0 => Some("0"),
        Keycode::_1 => Some("1"),
        Keycode::_2 => Some("2"),
        Keycode::_3 => Some("3"),
        Keycode::_4 => Some("4"),
        Keycode::_5 => Some("5"),
        Keycode::_6 => Some("6"),
        Keycode::_7 => Some("7"),
        Keycode::_8 => Some("8"),
        Keycode::_9 => Some("9"),
        Keycode::Kp1 => Some("Numpad1"),
        Keycode::Kp2 => Some("Numpad2"),
        Keycode::Kp3 => Some("Numpad3"),
        Keycode::Kp4 => Some("Numpad4"),
        Keycode::Kp5 => Some("Numpad5"),
        Keycode::Kp6 => Some("Numpad6"),
        Keycode::Kp7 => Some("Numpad7"),
        Keycode::Kp8 => Some("Numpad8"),
        Keycode::A => Some("A"),
        Keycode::B => Some("B"),
        Keycode::C => Some("C"),
        Keycode::D => Some("D"),
        Keycode::E => Some("E"),
        Keycode::F => Some("F"),
        Keycode::G => Some("G"),
        Keycode::H => Some("H"),
        Keycode::I => Some("I"),
        Keycode::J => Some("J"),
        Keycode::K => Some("K"),
        Keycode::L => Some("L"),
        Keycode::M => Some("M"),
        Keycode::N => Some("N"),
        Keycode::O => Some("O"),
        Keycode::P => Some("P"),
        Keycode::Q => Some("Q"),
        Keycode::R => Some("R"),
        Keycode::S => Some("S"),
        Keycode::T => Some("T"),
        Keycode::U => Some("U"),
        Keycode::V => Some("V"),
        Keycode::W => Some("W"),
        Keycode::X => Some("X"),
        Keycode::Y => Some("Y"),
        Keycode::Z => Some("Z"),
        _ => None,
    }
}

pub(super) fn track_indicator_target(
    kind: crate::ui::TrackIndicatorKind,
    overlay_slot: Option<Rect>,
) -> Option<DiscoverabilityTarget> {
    match kind {
        crate::ui::TrackIndicatorKind::Armed => Some(DiscoverabilityTarget {
            action: AppAction::ToggleCurrentTrackArm,
            display_scope: Some("Active Track"),
            allowed_mapping_scopes: &["Active Track"],
            overlay_slot,
        }),
        crate::ui::TrackIndicatorKind::Recording => Some(DiscoverabilityTarget {
            action: AppAction::ToggleRecording,
            display_scope: Some("Armed/Active"),
            allowed_mapping_scopes: &["Armed/Active", "Active Track"],
            overlay_slot,
        }),
        crate::ui::TrackIndicatorKind::Muted => Some(DiscoverabilityTarget {
            action: AppAction::ToggleCurrentTrackMute,
            display_scope: Some("Active Track"),
            allowed_mapping_scopes: &["Active Track"],
            overlay_slot,
        }),
        crate::ui::TrackIndicatorKind::Solo => Some(DiscoverabilityTarget {
            action: AppAction::ToggleCurrentTrackSolo,
            display_scope: Some("Active Track"),
            allowed_mapping_scopes: &["Active Track"],
            overlay_slot,
        }),
    }
}
