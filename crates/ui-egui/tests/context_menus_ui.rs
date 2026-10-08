//! Headless UI tests of the right-click menus (docs/menus.md): they hold Premiere Pro's rows in
//! Premiere's order, look like the other menus, and their rows work.
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

    /// Ids of the on-screen elements starting with `prefix`.
    fn ids(&mut self, prefix: &str) -> Vec<String> {
        self.ok("ui.elements", json!({"prefix": prefix})).as_array().unwrap().iter().filter_map(|e| e["id"].as_str().map(str::to_string)).collect()
    }

    /// The labels of the elements starting with `prefix`, top to bottom.
    fn rows(&mut self, prefix: &str) -> Vec<String> {
        let v = self.ok("ui.elements", json!({"prefix": prefix}));
        let mut rows: Vec<(f64, String)> =
            v.as_array().unwrap().iter().map(|e| (e["rect"][1].as_f64().unwrap_or(0.0), e["label"].as_str().unwrap_or("").to_string())).collect();
        rows.sort_by(|a, b| a.0.total_cmp(&b.0));
        rows.into_iter().map(|r| r.1).collect()
    }

    fn right_click(&mut self, id: &str) {
        self.ok("ui.click", json!({"id": id, "button": "right"}));
        self.frames(4);
    }

    fn escape(&mut self) {
        self.ok("ui.key", json!({"key": "Escape"}));
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
fn the_effect_controls_keyframe_menu_is_premieres() {
    let mut d = Driver::new();
    let clip = d
        .ids("timeline.clip.")
        .into_iter()
        .find(|i| i.trim_start_matches("timeline.clip.").chars().all(|c| c.is_ascii_digit()))
        .expect("a clip in the demo timeline");
    d.click(&clip);
    d.click("panel.tab.EffectControls");
    // animate Scale: a keyframe appears at the playhead
    let stopwatch = d.ids("effectControls.").into_iter().find(|i| i.ends_with(".scale.stopwatch")).expect("Motion ▸ Scale");
    d.click(&stopwatch);
    let lane = stopwatch.trim_end_matches("stopwatch").to_string();
    let key = d.ids(&format!("{lane}keyframe.")).into_iter().next().expect("the new keyframe");
    d.right_click(&key);
    let rows = d.rows(&format!("{key}."));
    assert_eq!(rows, ["Clear", "Linear", "Bezier", "Auto Bezier", "Continuous Bezier", "Hold", "Ease In", "Ease Out"], "the working rows, in Premiere's order");
    d.shot("context-effect-controls-keyframe");
    // a row sets the keyframe's interpolation, and the menu closes
    d.click(&format!("{key}.hold"));
    assert!(!d.has(&format!("{key}.hold")), "the menu closes");
    let interpolations: Vec<String> = d
        .harness
        .state()
        .session
        .active_sequence()
        .unwrap()
        .video_tracks
        .iter()
        .flat_map(|t| &t.items)
        .flat_map(|it| &it.effects)
        .filter_map(|e| e.params.get("scale"))
        .flat_map(|p| &p.keyframes)
        .map(|k| format!("{:?}", k.interp))
        .collect();
    assert_eq!(interpolations, ["Hold"], "the keyframe holds now");

    // the ruler of the panel has the marker and in/out rows
    d.right_click("effectControls.ruler");
    assert!(d.has("effectControls.ruler.markers.markIn") && d.has("effectControls.ruler.markers.add"));
    d.shot("context-effect-controls-ruler");
    d.escape();
}

#[test]
fn timeline_right_click_menus_depend_on_what_is_under_the_pointer() {
    let mut d = Driver::new();
    let clip = d
        .ids("timeline.clip.")
        .into_iter()
        .find(|i| i.trim_start_matches("timeline.clip.").chars().all(|c| c.is_ascii_digit()))
        .expect("a clip in the demo timeline");
    d.right_click(&clip);
    for id in ["edit.cut", "clip.nest", "clip.multicam", "clip.audioGain"] {
        assert!(d.has(&format!("timeline.clipMenu.{id}")), "no {id} in the clip menu");
    }
    assert!(!d.has("timeline.trackMenu.sequence.addTracks"));
    d.shot("context-timeline-clip");
    d.escape();

    // a track header has its own menu
    d.right_click("timeline.track.V1.header");
    assert!(d.has("timeline.trackMenu.sequence.addTracks") && d.has("timeline.trackMenu.rename"), "{:?}", d.ids("timeline.trackMenu."));
    assert!(!d.has("timeline.clipMenu.edit.cut"));
    d.shot("context-timeline-track-header");
    d.escape();
    assert!(!d.has("timeline.trackMenu.rename"), "Escape closes the menu");
}

#[test]
fn monitor_and_project_right_click_menus() {
    let mut d = Driver::new();
    d.right_click("program.picture");
    for id in ["markers.markIn", "markers.add", "sequence.lift"] {
        assert!(d.has(&format!("program.picture.menu.{id}")), "no {id} in the Program picture menu: {:?}", d.ids("program.picture.menu."));
    }
    d.shot("context-program-picture");
    d.escape();

    d.click("program.settings");
    d.shot("context-program-settings");
    d.escape();

    let item = d.ids("project.item.").into_iter().next();
    if let Some(item) = item {
        d.right_click(&item);
        assert!(d.has("project.itemMenu.rename") && d.has("project.itemMenu.duplicate"), "{:?}", d.ids("project.itemMenu."));
        d.shot("context-project-item");
        d.escape();
    }
}
