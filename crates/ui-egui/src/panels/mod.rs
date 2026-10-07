//! Panel bodies. `show` dispatches on [`PanelKind`]; drag-and-drop between panels (project items,
//! effects) is carried in egui temp data so the timeline/monitors can accept drops.

pub mod audio_fx_editor;
pub mod clip_dialogs;
pub mod color_dialogs;
pub mod dialogs;
pub mod effect_controls;
pub mod effects;
pub mod essential_sound;
pub mod events;
pub mod export_mode;
pub mod file_dialogs;
pub mod graphics;
pub mod graphics_templates;
pub mod import_mode;
pub mod interchange_export;
pub mod keyboard;
pub mod lumetri;
pub mod masks;
pub mod media_browser;
pub mod media_dialogs;
pub mod menu_dialogs;
pub mod metadata;
pub mod meters;
pub mod misc;
pub mod mixer;
pub mod monitor;
pub mod monitor_view;
pub mod multicam;
pub mod panel_state;
pub mod presets;
pub mod project;
pub mod project_dialogs;
pub mod project_views;
pub mod reference;
pub mod remix;
pub mod scopes;
pub mod settings;
pub mod shortcuts_dialog;
pub mod text;
pub mod timecode;
pub mod timeline;
pub mod timeline_automation;
pub mod timeline_captions;
pub mod tools;
pub mod trim_monitor;
pub mod voiceover;
pub mod workspaces;

use egui::{Align2, Color32, Rect};
use filmcraft_project::ItemId;

use crate::FilmcraftApp;
use crate::dock::PanelKind;
use crate::menu_layout::Entry::{self, Cmd, Rest, Sep, Sub, Tbd};
use crate::theme::Tokens;

pub fn show(app: &mut FilmcraftApp, ui: &mut egui::Ui, p: PanelKind, rect: Rect) {
    match p {
        PanelKind::Program if !app.session.state.edit_points.is_empty() => trim_monitor::show(app, ui, rect),
        PanelKind::Program => monitor::show(app, ui, rect, monitor::Which::Program),
        PanelKind::Source => monitor::show(app, ui, rect, monitor::Which::Source),
        PanelKind::Timeline => timeline::show(app, ui, rect),
        PanelKind::Project => project::show(app, ui, rect),
        PanelKind::Tools => tools::show(app, ui, rect),
        PanelKind::Effects => effects::show(app, ui, rect),
        PanelKind::EffectControls => effect_controls::show(app, ui, rect),
        PanelKind::AudioMeters => meters::show(app, ui, rect),
        PanelKind::LumetriColor => lumetri::show(app, ui, rect),
        PanelKind::Properties if graphics::graphic_selected(app) => graphics::properties(app, ui, rect),
        PanelKind::Properties => effect_controls::properties_panel(app, ui, rect),
        PanelKind::EssentialGraphics => graphics_templates::essential_graphics(app, ui, rect),
        PanelKind::History => misc::history(app, ui, rect),
        PanelKind::Markers => misc::markers(app, ui, rect),
        PanelKind::Info => misc::info(app, ui, rect),
        PanelKind::MediaBrowser => media_browser::show(app, ui, rect),
        PanelKind::AudioTrackMixer => mixer::track_mixer(app, ui, rect),
        PanelKind::AudioClipMixer => mixer::clip_mixer(app, ui, rect),
        PanelKind::LumetriScopes => scopes::show(app, ui, rect),
        PanelKind::Metadata => metadata::show(app, ui, rect),
        PanelKind::Timecode => timecode::show(app, ui, rect),
        PanelKind::Events => events::events(app, ui, rect),
        PanelKind::Progress => events::progress(app, ui, rect),
        PanelKind::ReferenceMonitor => reference::show(app, ui, rect),
        PanelKind::Text => text::show(app, ui, rect),
        PanelKind::EssentialSound => essential_sound::show(app, ui, rect),
        other => crate::dock::placeholder(ui, rect, &app.tokens, &format!("{} — coming in a later milestone", other.title())),
    }
}

#[derive(Clone, Debug)]
enum DragPayload {
    Item(ItemId),
    Effect(String),
    /// A graphics template (id, name) from Essential Graphics ▸ Browse.
    Template(String, String),
}

fn payload_id() -> egui::Id {
    egui::Id::new("filmcraft-drag-payload")
}

pub fn start_drag_item(ui: &egui::Ui, item: ItemId) {
    ui.ctx().data_mut(|d| d.insert_temp(payload_id(), Some(DragPayloadBox(DragPayload::Item(item)))));
}
pub fn start_drag_template(ui: &egui::Ui, id: &str, name: &str) {
    ui.ctx().data_mut(|d| d.insert_temp(payload_id(), Some(DragPayloadBox(DragPayload::Template(id.to_string(), name.to_string())))));
}
pub fn dragged_template(ui: &egui::Ui) -> Option<String> {
    match payload(ui) {
        Some(DragPayload::Template(id, _)) => Some(id),
        _ => None,
    }
}
pub fn start_drag_effect(ui: &egui::Ui, id: &str) {
    ui.ctx().data_mut(|d| d.insert_temp(payload_id(), Some(DragPayloadBox(DragPayload::Effect(id.to_string())))));
}
#[derive(Clone, Debug)]
struct DragPayloadBox(DragPayload);

fn payload(ui: &egui::Ui) -> Option<DragPayload> {
    ui.ctx().data(|d| d.get_temp::<Option<DragPayloadBox>>(payload_id())).flatten().map(|b| b.0)
}
pub fn dragged_project_item(ui: &egui::Ui) -> Option<ItemId> {
    match payload(ui) {
        Some(DragPayload::Item(i)) => Some(i),
        _ => None,
    }
}
pub fn dragged_effect(ui: &egui::Ui) -> Option<String> {
    match payload(ui) {
        Some(DragPayload::Effect(e)) => Some(e),
        _ => None,
    }
}
pub fn clear_drag(ui: &egui::Ui) {
    ui.ctx().data_mut(|d| d.insert_temp::<Option<DragPayloadBox>>(payload_id(), None));
}

/// Draw the drag ghost near the pointer and clear the payload after release.
pub fn drag_ghost(app: &FilmcraftApp, ui: &egui::Ui) {
    let Some(pl) = payload(ui) else { return };
    let ctx = ui.ctx();
    if let Some(p) = ctx.pointer_hover_pos() {
        let label = match &pl {
            DragPayload::Item(i) => app.session.project.item(*i).map(|x| x.name.clone()).unwrap_or_default(),
            DragPayload::Template(_, name) => name.clone(),
            DragPayload::Effect(e) => match e.strip_prefix("preset:") {
                Some(name) => name.to_string(),
                None => filmcraft_project::find_effect(e).map(|d| d.name.to_string()).unwrap_or_default(),
            },
        };
        let painter = ctx.layer_painter(egui::LayerId::new(egui::Order::Tooltip, egui::Id::new("drag-ghost")));
        let r = Rect::from_min_size(p + egui::vec2(12.0, 8.0), egui::vec2(label.len() as f32 * 7.0 + 16.0, 20.0));
        painter.rect_filled(r, 4.0, Color32::from_black_alpha(200));
        painter.text(r.center(), Align2::CENTER_CENTER, label, Tokens::ui(11.5), Color32::WHITE);
    }
    if ctx.input(|i| i.pointer.any_released()) {
        // cleared next frame so drop targets see the release this frame
        let f = ctx.cumulative_frame_nr();
        let k = egui::Id::new("drag-release-frame");
        match ctx.data(|d| d.get_temp::<u64>(k)) {
            Some(prev) if prev < f => {
                clear_drag(ui);
                ctx.data_mut(|d| d.remove::<u64>(k));
            }
            None => {
                ctx.data_mut(|d| d.insert_temp(k, f));
            }
            Some(_) => {}
        }
    } else if !ctx.input(|i| i.pointer.any_down()) {
        clear_drag(ui);
    }
}

/// What the menu of panel `p` holds below the entries every panel menu starts with, in Premiere
/// Pro's order (observed in Premiere Pro 2026). `Cmd` runs an engine command; `Tbd` is an entry
/// FilmCraft does not have yet. Panels not listed have only the common entries. The Project and
/// Media Browser panels draw their own (`project::panel_menu`, `media_browser::panel_menu`).
fn panel_layout(p: PanelKind) -> &'static [Entry] {
    match p {
        PanelKind::Program => &[Cmd("sequence.close"), Tbd("Close All")],
        PanelKind::Source => &[Tbd("Close"), Tbd("Close All")],
        PanelKind::EffectControls => &[
            Cmd("presets.save"),
            Sep,
            Tbd("Effect Enabled"),
            Sep,
            Cmd("effects.remove"),
            Tbd("Remove Effects…"),
            Sep,
            Tbd("Snap"),
            Tbd("Snap To"),
            Sep,
            Tbd("Show Audio Time Units"),
            Tbd("Time Ruler Numbers"),
            Tbd("Loop During Audio-Only Playback"),
            Sep,
            Tbd("Pin to Clip"),
            Sep,
            Tbd("Manage Audio Plug-Ins…"),
            Tbd("Manage Video Effects…"),
        ],
        PanelKind::Effects => &[
            Tbd("New Custom Bin"),
            Tbd("New Presets Bin"),
            Tbd("Delete Custom Item"),
            Sep,
            Cmd("effects.setDefaultTransition"),
            Tbd("Set Default Transition Duration…"),
            Sep,
            Cmd("presets.import"),
            Cmd("presets.export"),
            Tbd("Preset Properties…"),
            Sep,
            Tbd("Dynamic Lumetri Preset Previews"),
            Sep,
            Tbd("Manage Audio Plug-Ins…"),
            Tbd("Manage Video Effects…"),
        ],
        PanelKind::Timeline => &[
            Tbd("Work Area Bar"),
            Tbd("Show Audio Time Units"),
            Tbd("Audio Waveforms Use Label Color"),
            Tbd("Rectified Audio Waveforms"),
            Tbd("Logarithmic Waveform Scaling"),
            Tbd("Time Ruler Numbers"),
            Tbd("Start Time…"),
            Sep,
            Tbd("Video Head and Tail Thumbnails"),
            Tbd("Video Head Thumbnails"),
            Tbd("Continuous Video Thumbnails"),
            Sep,
            Tbd("Create Preset from Sequence…"),
            Cmd("sequence.revealInProject"),
            Cmd("media.linkMedia"),
            Cmd("media.makeOffline"),
            Sep,
            Cmd("multicam.audioFollowsVideo"),
            Cmd("multicam.selectionTopDown"),
            Tbd("Multi-Camera Follows Nest Setting"),
            Sep,
            Tbd("Label"),
        ],
        _ => &[],
    }
}

/// Draw `entries` of panel `p`'s menu. Returns true when a command ran.
fn panel_entries(app: &mut FilmcraftApp, ui: &mut egui::Ui, p: PanelKind, entries: &[Entry]) -> bool {
    let mut ran = false;
    for e in entries {
        match e {
            Sep => crate::menus::separator(ui),
            Tbd(label) => {
                crate::menus::tbd(ui, label);
            }
            Cmd(id) => {
                let Some(spec) = filmcraft_engine::command_specs().iter().find(|c| c.id == *id) else { continue };
                let shortcut = app.session.shortcuts.primary(id);
                let r = crate::menus::entry(ui, spec.label, shortcut.as_deref(), app.session.is_enabled(id), false);
                app.auto.add(&format!("panel.menu.{}.{id}", p.id()), r.rect, spec.label);
                if r.clicked() {
                    if let Err(err) = app.session.execute(id, serde_json::json!({})) {
                        app.ui.status = err.to_string();
                    }
                    ran = true;
                }
            }
            Sub(name, inner) => {
                ui.menu_button(crate::menus::row_label(name), |ui| {
                    ui.set_min_width(200.0);
                    ran |= panel_entries(app, ui, p, inner);
                });
            }
            Rest => {}
        }
    }
    ran
}

/// The panel "≡" menu: the entries every panel has (as Premiere Pro's panel menus start), then the
/// panel's own.
pub fn panel_menu_popup(app: &mut FilmcraftApp, ui: &mut egui::Ui) {
    drag_ghost(app, ui);
    // bins opened in new windows (Project panel)
    project::floating(app, ui.ctx());
    let id = egui::Id::new("panel-menu");
    let Some((p, pos)) = ui.ctx().data(|d| d.get_temp::<(PanelKind, egui::Pos2)>(id)) else { return };
    let mut close = false;
    // a menu like those of the menu bar (same look, submenus open beside it); closing is done below
    let popup =
        egui::Popup::new(id.with("popup"), ui.ctx().clone(), egui::PopupAnchor::Position(pos), egui::LayerId::new(egui::Order::Foreground, id.with("layer")))
            .kind(egui::PopupKind::Menu)
            .layout(egui::Layout::top_down_justified(egui::Align::Min))
            .style(crate::theme::menu_style)
            .close_behavior(egui::PopupCloseBehavior::IgnoreClicks)
            .open(true)
            .show(|ui| {
                ui.set_min_width(230.0);
                let key = |k: &str| format!("panel.menu.{}.{k}", p.id());
                // the Timeline's tabs are its open sequences: Close Panel closes the active one and
                // keeps the panel (Premiere's wording and behaviour)
                let timeline = p == PanelKind::Timeline && app.session.state.active_sequence.is_some();
                let r = crate::menus::entry(ui, "Close Panel", None, true, false);
                app.auto.add(&key("close"), r.rect, "Close Panel");
                if r.clicked() {
                    if timeline {
                        let _ = app.session.execute("sequence.close", serde_json::json!({}));
                    } else {
                        app.ui.dock.close(p);
                    }
                    close = true;
                }
                crate::menus::tbd(ui, "Undock Panel");
                crate::menus::tbd(ui, "Close Other Panels in Group");
                if timeline {
                    let r = crate::menus::entry(ui, "Close Other Timeline Panels", None, app.session.state.open_sequences.len() > 1, false);
                    app.auto.add(&key("closeOthers"), r.rect, "Close Other Timeline Panels");
                    if r.clicked() {
                        let _ = app.session.execute("sequence.closeOthers", serde_json::json!({}));
                        close = true;
                    }
                }
                let group = ui.menu_button(crate::menus::row_label("Panel Group Settings"), |ui| {
                    ui.set_min_width(200.0);
                    crate::menus::tbd(ui, "Close Panel Group");
                    crate::menus::tbd(ui, "Undock Panel Group");
                    let r = crate::menus::entry(ui, "Maximize Panel Group", None, true, false);
                    app.auto.add(&key("maximize"), r.rect, "Maximize Panel Group");
                    if r.clicked() {
                        app.ui.dock = crate::dock::DockNode::Tabs { panels: vec![p], active: 0 };
                        close = true;
                    }
                    // the way back from a maximized group (Premiere toggles Maximize instead)
                    let r = crate::menus::entry(ui, "Restore Workspace", None, true, false);
                    app.auto.add(&key("restoreWorkspace"), r.rect, "Restore Workspace");
                    if r.clicked() {
                        let w = app.ui.workspace.clone();
                        app.set_workspace(&w);
                        close = true;
                    }
                    crate::menus::separator(ui);
                    crate::menus::tbd(ui, "Stacked Panel Group");
                    crate::menus::tbd(ui, "Solo Panels in Stack");
                    crate::menus::tbd(ui, "Small Tabs");
                });
                app.auto.add(&key("groupSettings"), group.response.rect, "Panel Group Settings");
                let own = panel_layout(p);
                if !own.is_empty() {
                    crate::menus::separator(ui);
                    close |= panel_entries(app, ui, p, own);
                }
                if p == PanelKind::Timeline {
                    crate::menus::separator(ui);
                    for (k, label, on) in
                        [("thumbnails", "Video Thumbnails", app.ui.timeline.show_thumbnails), ("waveforms", "Audio Waveforms", app.ui.timeline.show_waveforms)]
                    {
                        let r = crate::menus::entry(ui, label, None, true, on);
                        app.auto.add(&key(k), r.rect, label);
                        if r.clicked() {
                            match k {
                                "thumbnails" => app.ui.timeline.show_thumbnails = !on,
                                _ => app.ui.timeline.show_waveforms = !on,
                            }
                        }
                    }
                }
                if p == PanelKind::Project {
                    crate::menus::separator(ui);
                    close |= project::panel_menu(app, ui);
                }
                if p == PanelKind::MediaBrowser {
                    crate::menus::separator(ui);
                    close |= media_browser::panel_menu(app, ui);
                }
            });
    let outside = popup.as_ref().is_some_and(|r| r.response.clicked_elsewhere());
    // A click elsewhere or Escape closes the menu. The click that opened it is over the tab, not
    // the menu, and must not close it again in the same frame.
    let fresh = ui.ctx().data(|d| d.get_temp::<u64>(egui::Id::new("panel-menu-opened"))) == Some(ui.ctx().cumulative_frame_nr());
    if close || (!fresh && outside) || ui.input(|i| i.key_pressed(egui::Key::Escape)) {
        ui.ctx().data_mut(|d| d.remove::<(PanelKind, egui::Pos2)>(id));
    }
}
