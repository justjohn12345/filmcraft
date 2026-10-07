//! Native macOS menu bar (muda): like Premiere, the menus live in the system menu bar. They are
//! laid out by `filmcraft_ui_egui::menu_layout`, as the in-window menu bar of the other platforms
//! is: Premiere's order and separators, "(TBD)" rows for what is not there yet. Items send command
//! ids to the app's command inbox, and follow the commands' enabled and checked state.

use std::sync::mpsc::{Receiver, channel};

use filmcraft_ui_egui::FilmcraftApp;
use filmcraft_ui_egui::menu_layout::{Node, TBD, menu};
use filmcraft_ui_egui::menus::{MENUS, MenuItem as Item, menu_items};
use muda::accelerator::Accelerator;
use muda::{CheckMenuItem, Menu, MenuEvent, MenuItem, PredefinedMenuItem, Submenu};

fn accel(s: &str) -> Option<Accelerator> {
    // Only shortcuts with a modifier become native key equivalents (single keys stay in-app so
    // typing in fields keeps working).
    if !(s.contains("Cmd+") || s.contains("Ctrl+") || s.contains("Alt+")) {
        return None;
    }
    let mapped =
        s.replace("Cmd+", "CmdOrCtrl+").replace(";", "Semicolon").replace("'", "Quote").replace("/", "Slash").replace("=", "Equal").replace("\\", "Backslash");
    mapped.parse().ok()
}

/// Brings the native menu bar up to date with the menu commands: labels, key equivalents, which
/// items are enabled and which are checked.
pub type ShortcutUpdater = Box<dyn FnMut(&[Item])>;

/// A label as AppKit shows it: a single `&` would be taken as a mnemonic mark and dropped.
fn text(label: &str) -> String {
    label.replace('&', "&&")
}

/// A native item that follows a command.
enum Native {
    Plain(MenuItem),
    Check(CheckMenuItem),
}

/// What a native item last showed, so that only changes are sent to AppKit.
#[derive(PartialEq)]
struct Shown {
    label: String,
    shortcut: Option<String>,
    enabled: bool,
    checked: bool,
}

impl Shown {
    fn of(it: &Item) -> Self {
        Self { label: it.label.clone(), shortcut: it.shortcut.clone(), enabled: it.enabled, checked: it.checked == Some(true) }
    }
}

struct Built {
    native: Vec<(String, Native, Shown)>,
    titles: Vec<(String, Submenu)>,
}

/// Commands of the application menu (About, Settings, Keyboard Shortcuts), as in Premiere on
/// macOS; the in-window menu bar of other platforms has them in Edit and Help.
fn in_app_menu(id: &str) -> bool {
    id.starts_with("app.settings.") || id == "app.keyboardShortcuts" || id == "app.about"
}

fn item(it: &Item, built: &mut Built) -> Native {
    let accelerator = it.shortcut.as_deref().and_then(accel);
    let n = match it.checked {
        Some(on) => Native::Check(CheckMenuItem::with_id(it.id.clone(), text(&it.label), it.enabled, on, accelerator)),
        None => Native::Plain(MenuItem::with_id(it.id.clone(), text(&it.label), it.enabled, accelerator)),
    };
    let twin = match &n {
        Native::Check(c) => Native::Check(c.clone()),
        Native::Plain(p) => Native::Plain(p.clone()),
    };
    built.native.push((it.id.clone(), twin, Shown::of(it)));
    n
}

/// Append the laid-out `nodes` (see `menu_layout`) to `parent`: commands, separators between
/// groups, "(TBD)" rows for what is not there yet, submenus. Returns how many rows were added.
fn fill(parent: &Submenu, nodes: &[Node<'_>], language: filmcraft_ui_egui::i18n::Language, built: &mut Built) -> usize {
    let mut rows = 0;
    let mut separator = false;
    let row = |parent: &Submenu, separator: &mut bool, rows: &mut usize, it: &dyn muda::IsMenuItem| {
        if *separator {
            let _ = parent.append(&PredefinedMenuItem::separator());
            *separator = false;
        }
        let _ = parent.append(it);
        *rows += 1;
    };
    for node in nodes {
        match node {
            Node::Sep => separator = rows > 0,
            Node::Item(it) if in_app_menu(&it.id) => {}
            Node::Item(it) => match item(it, built) {
                Native::Check(c) => row(parent, &mut separator, &mut rows, &c),
                Native::Plain(p) => row(parent, &mut separator, &mut rows, &p),
            },
            Node::Tbd(label) => row(parent, &mut separator, &mut rows, &MenuItem::new(text(&format!("{} {}", language.tr(label), TBD)), false, None)),
            Node::Sub(name, kids) => {
                let sub = Submenu::new(text(language.tr(name)), true);
                if fill(&sub, kids, language, built) > 0 {
                    built.titles.push((name.clone(), sub.clone()));
                    row(parent, &mut separator, &mut rows, &sub);
                }
            }
        }
    }
    rows
}

pub fn install(app: &FilmcraftApp, ctx: egui::Context) -> (Receiver<String>, ShortcutUpdater) {
    let items = menu_items(app);
    let language = app.ui.language;
    let mut built = Built { native: Vec::new(), titles: Vec::new() };
    let bar = Menu::new();
    let app_menu = Submenu::new("FilmCraft", true);
    // FilmCraft ▸ Settings ▸ <category> (Premiere's app-menu layout; General is Cmd+,)
    let settings = Submenu::new(language.tr("Settings"), true);
    built.titles.push(("Settings".into(), settings.clone()));
    for it in items.iter().filter(|i| i.id.starts_with("app.settings.")) {
        let label = it.label.trim_end_matches('…').to_string();
        let mi = MenuItem::with_id(it.id.clone(), text(&label), true, it.shortcut.as_deref().and_then(accel));
        built.native.push((it.id.clone(), Native::Plain(mi.clone()), Shown { label, ..Shown::of(it) }));
        let _ = settings.append(&mi);
    }
    let _ = app_menu.append_items(&[
        &MenuItem::with_id("app.about", "About FilmCraft", true, None),
        &MenuItem::with_id("help.discord", "Join the ArtCraft Discord…", true, None),
    ]);
    let _ = app_menu.append_items(&[&PredefinedMenuItem::separator(), &settings, &PredefinedMenuItem::separator()]);
    if let Some(it) = items.iter().find(|i| i.id == "app.keyboardShortcuts") {
        match item(it, &mut built) {
            Native::Check(c) => drop(app_menu.append(&c)),
            Native::Plain(p) => drop(app_menu.append(&p)),
        }
        let _ = app_menu.append(&PredefinedMenuItem::separator());
    }
    let _ = app_menu.append_items(&[
        &PredefinedMenuItem::services(None),
        &PredefinedMenuItem::separator(),
        &PredefinedMenuItem::hide(None),
        &PredefinedMenuItem::hide_others(None),
        &PredefinedMenuItem::show_all(None),
        &PredefinedMenuItem::separator(),
        &PredefinedMenuItem::quit(None),
    ]);
    let _ = bar.append(&app_menu);
    // the nine menus, laid out as the in-window menu bar is (Premiere's order and separators)
    for top in MENUS {
        let sub = Submenu::new(language.tr(top), true);
        built.titles.push((top.into(), sub.clone()));
        fill(&sub, &menu(top, &items), language, &mut built);
        let _ = bar.append(&sub);
    }
    bar.init_for_nsapp();
    if let Some(help) = bar.items().last().and_then(|i| i.as_submenu().cloned()) {
        help.set_as_help_menu_for_nsapp();
    }
    Box::leak(Box::new(bar));
    let (tx, rx) = channel();
    std::thread::spawn(move || {
        while let Ok(ev) = MenuEvent::receiver().recv() {
            if tx.send(ev.id().0.clone()).is_err() {
                break;
            }
            ctx.request_repaint();
        }
    });
    let Built { mut native, titles } = built;
    let mut shown_language = language;
    let update: ShortcutUpdater = Box::new(move |items: &[Item]| {
        let checked = |id: &str| items.iter().any(|it| it.id == id && it.checked == Some(true));
        let language = if checked("app.language.japanese") {
            filmcraft_ui_egui::i18n::Language::Ja
        } else if checked("app.language.spanish") {
            filmcraft_ui_egui::i18n::Language::Es
        } else {
            filmcraft_ui_egui::i18n::Language::En
        };
        if language != shown_language {
            shown_language = language;
            for (title, submenu) in &titles {
                submenu.set_text(text(language.tr(title)));
            }
        }
        for (id, mi, shown) in &mut native {
            let Some(it) = items.iter().find(|i| &i.id == id) else { continue };
            let mut now = Shown::of(it);
            if id.starts_with("app.settings.") {
                // Settings ▸ General, not General…
                now.label = now.label.trim_end_matches('…').to_string();
                now.enabled = true;
            }
            if now == *shown {
                continue;
            }
            match mi {
                Native::Plain(m) => {
                    m.set_text(text(&now.label));
                    m.set_enabled(now.enabled);
                    let _ = m.set_accelerator(now.shortcut.as_deref().and_then(accel));
                }
                Native::Check(m) => {
                    m.set_text(text(&now.label));
                    m.set_enabled(now.enabled);
                    m.set_checked(now.checked);
                    let _ = m.set_accelerator(now.shortcut.as_deref().and_then(accel));
                }
            }
            *shown = now;
        }
    });
    (rx, update)
}
