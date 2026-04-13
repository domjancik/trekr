use crate::launcher::ui::pages::LauncherPage;
use sdl3::event::Event;
use sdl3::keyboard::{Keycode, Mod};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LauncherUiAction {
    Quit,
    ShowPage(LauncherPage),
    ShowNextPage,
    ShowPreviousPage,
    SelectPreviousItem,
    SelectNextItem,
    AdjustBackward,
    AdjustForward,
    ActivateItem,
    InstallOrUpdate,
    DeleteInstall,
    RefreshBranches,
}

pub fn resolve_keyboard(event: &Event) -> Option<LauncherUiAction> {
    match event {
        Event::Quit { .. } => Some(LauncherUiAction::Quit),
        Event::KeyDown {
            keycode: Some(Keycode::Escape),
            ..
        } => Some(LauncherUiAction::Quit),
        Event::KeyDown {
            keycode: Some(Keycode::Tab),
            keymod,
            repeat: false,
            ..
        } => Some(if keymod.intersects(Mod::LSHIFTMOD | Mod::RSHIFTMOD) {
            LauncherUiAction::ShowPreviousPage
        } else {
            LauncherUiAction::ShowNextPage
        }),
        Event::KeyDown {
            keycode: Some(Keycode::F1),
            repeat: false,
            ..
        } => Some(LauncherUiAction::ShowPage(LauncherPage::Launch)),
        Event::KeyDown {
            keycode: Some(Keycode::F2),
            repeat: false,
            ..
        } => Some(LauncherUiAction::ShowPage(LauncherPage::Branches)),
        Event::KeyDown {
            keycode: Some(Keycode::F3),
            repeat: false,
            ..
        } => Some(LauncherUiAction::ShowPage(LauncherPage::Installs)),
        Event::KeyDown {
            keycode: Some(Keycode::F4),
            repeat: false,
            ..
        } => Some(LauncherUiAction::ShowPage(LauncherPage::Settings)),
        Event::KeyDown {
            keycode: Some(Keycode::Up),
            repeat: false,
            ..
        } => Some(LauncherUiAction::SelectPreviousItem),
        Event::KeyDown {
            keycode: Some(Keycode::Down),
            repeat: false,
            ..
        } => Some(LauncherUiAction::SelectNextItem),
        Event::KeyDown {
            keycode: Some(Keycode::Q),
            repeat: false,
            ..
        } => Some(LauncherUiAction::AdjustBackward),
        Event::KeyDown {
            keycode: Some(Keycode::E),
            repeat: false,
            ..
        } => Some(LauncherUiAction::AdjustForward),
        Event::KeyDown {
            keycode: Some(Keycode::R),
            repeat: false,
            ..
        } => Some(LauncherUiAction::RefreshBranches),
        Event::KeyDown {
            keycode: Some(Keycode::Return),
            repeat: false,
            ..
        } => Some(LauncherUiAction::ActivateItem),
        Event::KeyDown {
            keycode: Some(Keycode::U),
            repeat: false,
            ..
        } => Some(LauncherUiAction::InstallOrUpdate),
        Event::KeyDown {
            keycode: Some(Keycode::Delete),
            repeat: false,
            ..
        } => Some(LauncherUiAction::DeleteInstall),
        _ => None,
    }
}
