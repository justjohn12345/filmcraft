//! Headless UI tests of the menu bar and the panel menus (docs/menus.md): the bar is laid out as
//! `menu_layout` says (Premiere Pro's order, separators, "(TBD)" for what is not there yet), its
//! items work, and every panel menu starts with the same entries.
//!
//! With `FILMCRAFT_UI_SHOTS=<dir>` the tests also render the UI with wgpu and save PNGs there.

use std::sync::mpsc::{Sender, channel};

use egui_kittest::Harness;
use filmcraft_engine::Session;
use filmcraft_ui_egui::FilmcraftApp;
use filmcraft_ui_egui::control::ControlRequest;
use serde_json::{Value, json};

struct Driver {
    harness: Harness<'static, FilmcraftApp>,
    tx: Sender<ControlRequest>,
    shots: Option<std::path::PathBuf>,
}

impl Driver {
    fn new() -> Self {
        let mut s = Session::default();
        s.execute("file.openDemoProject", json!({})).unwrap();
        let (tx, rx) = channel();
        let app = FilmcraftApp::new(s).with_control(rx);
        let shots = std::env::var_os("FILMCRAFT_UI_SHOTS").map(std::path::PathBuf::from);
        let mut b = Harness::builder().with_size(egui::vec2(1600.0, 980.0)).with_step_dt(1.0 / 60.0).with_max_steps(10_000);
        if shots.is_some() {
            b = b.wgpu().with_pixels_per_point(1.0);
        }
        let harness = b.build_eframe(move |_cc| app);
        let mut d = Driver { harness, tx, shots };
        d.frames(4);
        d
    }

    fn frames(&mut self, n: usize) {
        for _ in 0..n {
            let ctx = self.harness.ctx.clone();
            let mut raw = std::mem::take(self.harness.input_mut());
            eframe::App::raw_input_hook(self.harness.state_mut(), &ctx, &mut raw);
            *self.harness.input_mut() = raw;
            self.harness.step();
        }
    }

    fn ok(&mut self, method: &str, params: Value) -> Value {
        let (req, reply) = ControlRequest::new(method, params.clone());
        self.tx.send(req).unwrap();
        for _ in 0..600 {
            self.frames(1);
            if let Ok(v) = reply.try_recv() {
                assert_eq!(v["ok"], json!(true), "{method} {params} failed: {v}");
                return v["result"].clone();
            }
        }
        panic!("no reply to {method} {params}");
    }

    fn click(&mut self, id: &str) {
        self.ok("ui.click", json!({"id": id}));
        self.frames(4);
    }

    fn has(&mut self, id: &str) -> bool {
        self.ok("ui.elements", json!({"prefix": id})).as_array().unwrap().iter().any(|e| e["id"] == json!(id))
    }

    fn shot(&mut self, name: &str) {
        let Some(dir) = self.shots.clone() else { return };
        self.frames(8);
        let img = self.harness.render().expect("wgpu render");
        std::fs::create_dir_all(&dir).unwrap();
        img.save(dir.join(format!("{name}.png"))).unwrap();
    }
}

#[test]
fn the_menu_bar_opens_laid_out_menus_and_their_items_work() {
    let mut d = Driver::new();
    d.click("menubar.Sequence");
    assert!(d.has("menu.sequence.settings") && d.has("menu.sequence.addTracks"), "the Sequence menu is open");
    d.shot("menu-sequence");
    // a submenu opens beside its parent
    d.ok("ui.move", json!({"id": "menu.submenu.GotoGap"}));
    d.frames(30);
    d.shot("menu-sequence-submenu");
    d.ok("ui.key", json!({"key": "Escape"}));
    d.frames(4);

    d.click("menubar.Edit");
    d.shot("menu-edit");
    d.ok("ui.key", json!({"key": "Escape"}));
    d.frames(4);
    d.click("menubar.File");
    d.shot("menu-file");
    assert!(d.has("menu.file.save"));
    d.ok("ui.key", json!({"key": "Escape"}));
    d.frames(4);
    assert!(!d.has("menu.file.save"), "Escape closes the menu");

    // an item runs its command and closes the menu: Sequence ▸ Snap in Timeline toggles snapping
    let snapping = |d: &mut Driver| d.ok("ui.menu.tree", json!({})).to_string().contains(r#""checked":true,"enabled":true,"id":"sequence.snap""#);
    let before = snapping(&mut d);
    d.click("menubar.Sequence");
    d.click("menu.sequence.snap");
    assert!(!d.has("menu.sequence.snap"), "the menu closes");
    assert_ne!(snapping(&mut d), before, "and its checkmark follows");

    // the tree the control channel reports is the one on screen
    let tree = d.ok("ui.menu.tree", json!({}));
    let seq = tree.as_array().unwrap().iter().find(|m| m["label"] == "Sequence").unwrap()["items"].as_array().unwrap().clone();
    assert_eq!(seq[0]["id"], "sequence.settings");
    assert_eq!(seq[1], json!({"separator": true}));
    assert!(seq.iter().any(|n| n["tbd"] == true), "what is not there yet is marked: {seq:?}");
}

#[test]
fn panel_menus_start_alike_and_hold_the_panels_own_entries() {
    let mut d = Driver::new();
    // every panel menu starts with the same entries
    d.click("panel.menu.Program");
    assert!(d.has("panel.menu.Program.close") && d.has("panel.menu.Program.groupSettings"));
    assert!(d.has("panel.menu.Program.sequence.close"), "the Program panel's own entry");
    d.shot("panel-menu-program");
    d.ok("ui.key", json!({"key": "Escape"}));
    d.frames(4);
    assert!(!d.has("panel.menu.Program.close"), "Escape closes the menu");

    d.click("panel.menu.Timeline");
    for id in ["close", "closeOthers", "groupSettings", "sequence.revealInProject", "media.linkMedia", "multicam.audioFollowsVideo", "thumbnails", "waveforms"]
    {
        assert!(d.has(&format!("panel.menu.Timeline.{id}")), "no {id} in the Timeline panel menu");
    }
    d.shot("panel-menu-timeline");
    // a checked entry toggles and the menu stays open
    let thumbs = |d: &mut Driver| d.ok("ui.inspect", json!({}))["ui"]["timeline"]["show_thumbnails"].clone();
    let before = thumbs(&mut d);
    d.click("panel.menu.Timeline.thumbnails");
    assert_ne!(thumbs(&mut d), before);
    d.click("panel.menu.Timeline.thumbnails");
    assert_eq!(thumbs(&mut d), before);
    // the submenu every panel has
    d.ok("ui.move", json!({"id": "panel.menu.Timeline.groupSettings"}));
    d.frames(30);
    assert!(d.has("panel.menu.Timeline.maximize"), "Panel Group Settings opens beside the menu");
    d.shot("panel-menu-group-settings");
    d.ok("ui.key", json!({"key": "Escape"}));
    d.frames(4);

    d.click("panel.menu.Project");
    assert!(d.has("panel.menu.Project.close") && d.has("project.menu.newBin"));
    d.shot("panel-menu-project");
}
