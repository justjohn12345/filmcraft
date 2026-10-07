//! The menu bar's layout: which command goes where in each menu, and where the separator lines
//! are. It follows Premiere Pro's menus (observed in Premiere Pro 2026 on macOS), so that an editor
//! finds every item in its usual place.
//!
//! - [`Entry::Cmd`] places a registered command by id; its label, shortcut and state come from
//!   the registry as before.
//! - [`Entry::Tbd`] is an item Premiere has and FilmCraft does not yet: it is shown disabled and
//!   marked "(TBD)", so the gap is visible where the item will go.
//! - A command that is registered with a menu path but not named here is not lost: it goes to
//!   the end of its menu after a separator, or to where the menu says [`Entry::Rest`].
//!
//! Left out on purpose: items that only make sense with another vendor's products or services,
//! Productions and Team Projects (out of scope, see ROADMAP.md), the items macOS adds to every
//! Edit menu, and account items. Premiere's application menu
//! (About, Settings, Keyboard Shortcuts) has no place in an in-window menu bar; those commands
//! are at the end of Edit and Help, where Premiere has them on Windows.

use std::collections::HashSet;
use std::sync::OnceLock;

use crate::menus::MenuItem;

/// One line of a menu's layout.
pub enum Entry {
    /// A separator line.
    Sep,
    /// The command with this id (left out while no such command is registered).
    Cmd(&'static str),
    /// An item FilmCraft does not have yet: shown disabled and marked "(TBD)".
    Tbd(&'static str),
    /// A submenu.
    Sub(&'static str, &'static [Entry]),
    /// Where the commands of this menu that the layout does not name go (the end otherwise).
    Rest,
}
use Entry::{Cmd, Rest, Sep, Sub, Tbd};

/// One line of a menu as it is shown.
#[derive(Debug)]
pub enum Node<'a> {
    Sep,
    Item(&'a MenuItem),
    Tbd(&'static str),
    Sub(String, Vec<Node<'a>>),
}

/// The suffix of an item that is not there yet.
pub const TBD: &str = "(TBD)";

const FILE: &[Entry] = &[
    Sub(
        "New",
        &[
            Cmd("file.newProject"),
            Cmd("file.newSequence"),
            Cmd("file.newSequenceFromClip"),
            Cmd("file.newBin"),
            Cmd("file.newBinFromSelection"),
            Cmd("file.newSearchBin"),
            Cmd("file.newOfflineFile"),
            Cmd("file.newAdjustmentLayer"),
            Sep,
            Cmd("file.newBarsAndTone"),
            Cmd("file.newBlackVideo"),
            Cmd("file.newColorMatte"),
            Cmd("file.newTransparentVideo"),
            Cmd("file.newCountingLeader"),
        ],
    ),
    Cmd("file.open"),
    Tbd("Open Recent"),
    Sep,
    Cmd("file.close"),
    Cmd("file.closeProject"),
    Cmd("file.closeAllProjects"),
    Cmd("file.closeAllOtherProjects"),
    Cmd("file.save"),
    Cmd("file.saveAs"),
    Cmd("file.saveCopy"),
    Cmd("file.saveAsTemplate"),
    Cmd("file.saveAll"),
    Cmd("file.revert"),
    Sep,
    Cmd("media.linkMedia"),
    Cmd("media.makeOffline"),
    Sep,
    Cmd("file.importFromMediaBrowser"),
    Cmd("file.import"),
    Tbd("Import Recent File"),
    Sep,
    Sub(
        "Export",
        &[
            Cmd("file.exportMedia"),
            Cmd("file.exportGraphicsTemplate"),
            Cmd("captions.export"),
            Cmd("file.exportEdl"),
            Cmd("file.exportOmf"),
            Tbd("Markers…"),
            Cmd("file.exportSelectionProject"),
            Cmd("file.exportAaf"),
            Cmd("file.exportAle"),
            Cmd("file.exportOtio"),
            Cmd("file.exportFcp7Xml"),
        ],
    ),
    Sep,
    Sub("Get Media File Properties for", &[Cmd("file.mediaPropertiesFile"), Cmd("file.mediaProperties")]),
    Sep,
    Sub("Project Settings", &[Cmd("file.projectSettings.general"), Tbd("Color…"), Cmd("file.projectSettings.scratchDisks"), Cmd("project.ingestSettings")]),
    Sep,
    Cmd("file.projectManager"),
];

const EDIT: &[Entry] = &[
    Cmd("edit.undo"),
    Cmd("edit.redo"),
    Sep,
    Cmd("edit.cut"),
    Cmd("edit.copy"),
    Cmd("edit.paste"),
    Cmd("edit.pasteInsert"),
    Cmd("edit.pasteAttributes"),
    Cmd("edit.removeAttributes"),
    Cmd("edit.clear"),
    Cmd("edit.rippleDelete"),
    Sep,
    Cmd("edit.duplicate"),
    Cmd("edit.selectAll"),
    Cmd("edit.selectAllMatching"),
    Cmd("edit.deselectAll"),
    Sep,
    Cmd("edit.find"),
    Cmd("edit.findNext"),
    Tbd("Spelling"),
    Sep,
    Sub(
        "Label",
        &[
            Cmd("edit.selectLabelGroup"),
            Sep,
            Cmd("edit.label.violet"),
            Cmd("edit.label.iris"),
            Cmd("edit.label.caribbean"),
            Cmd("edit.label.lavender"),
            Cmd("edit.label.cerulean"),
            Cmd("edit.label.forest"),
            Cmd("edit.label.rose"),
            Cmd("edit.label.mango"),
            Cmd("edit.label.purple"),
            Cmd("edit.label.blue"),
            Cmd("edit.label.teal"),
            Cmd("edit.label.magenta"),
            Cmd("edit.label.tan"),
            Cmd("edit.label.green"),
            Cmd("edit.label.brown"),
            Cmd("edit.label.yellow"),
        ],
    ),
    Sep,
    Cmd("edit.removeUnused"),
    Cmd("edit.consolidateDuplicates"),
    Tbd("Generate Source Clips for Media"),
    Tbd("Reassociate Source Clips…"),
    Sep,
    Cmd("edit.editOriginal"),
];

const CLIP: &[Entry] = &[
    Cmd("clip.rename"),
    Cmd("clip.makeSubclip"),
    Cmd("clip.editSubclip"),
    Cmd("clip.editOffline"),
    Cmd("clip.sourceSettings"),
    Sep,
    Sub("Modify", &[Cmd("clip.audioChannels"), Tbd("Color…"), Cmd("clip.interpretFootage"), Cmd("clip.modifyTimecode"), Tbd("VR Properties…")]),
    Sub(
        "Video Options",
        &[
            Cmd("clip.frameHoldOptions"),
            Cmd("clip.frameHold"),
            Cmd("clip.insertFrameHoldSegment"),
            Cmd("clip.fieldOptions"),
            Sub(
                "Time Interpolation",
                &[Cmd("clip.timeInterpolation.frameSampling"), Cmd("clip.timeInterpolation.frameBlending"), Cmd("clip.timeInterpolation.opticalFlow")],
            ),
            Cmd("clip.scaleToFrameSize"),
            Cmd("clip.fitToFrame"),
            Cmd("clip.fillFrame"),
        ],
    ),
    Sub("Audio Options", &[Cmd("clip.audioGain"), Cmd("clip.breakoutToMono"), Cmd("clip.extractAudio")]),
    Sep,
    Cmd("clip.speedDuration"),
    Cmd("clip.sceneEditDetection"),
    Tbd("Enable Enhance Speech"),
    Sub("Remix", &[Cmd("clip.remix.enable"), Cmd("clip.remix.properties"), Cmd("clip.remix.revert")]),
    Sep,
    Cmd("source.insert"),
    Cmd("source.overwrite"),
    Sep,
    Tbd("Replace Footage…"),
    Sub("Replace With Clip", &[Cmd("clip.replaceFromSource"), Cmd("clip.replaceFromSourceMatchFrame"), Cmd("clip.replaceFromBin")]),
    Tbd("Render and Replace…"),
    Tbd("Restore Unrendered"),
    Cmd("clip.restoreCaptionsFromSource"),
    Cmd("clip.updateMetadata"),
    Sep,
    Cmd("clip.generateAudioWaveform"),
    Tbd("Auto-Tag Audio Types"),
    Sep,
    Cmd("clip.automateToSequence"),
    Sep,
    Cmd("clip.enable"),
    Cmd("clip.link"),
    Cmd("clip.group"),
    Cmd("clip.ungroup"),
    Cmd("clip.synchronize"),
    Cmd("clip.mergeClips"),
    Cmd("clip.nest"),
    Cmd("clip.createMulticam"),
    Sub("Multi-Camera", &[Cmd("clip.multicamEnable"), Cmd("clip.multicamFlatten")]),
];

const SEQUENCE: &[Entry] = &[
    Cmd("sequence.settings"),
    Sep,
    Cmd("sequence.renderEffectsInToOut"),
    Cmd("sequence.renderInToOut"),
    Cmd("sequence.renderSelection"),
    Cmd("sequence.renderAudio"),
    Cmd("sequence.deleteRenderFiles"),
    Cmd("sequence.deleteRenderFilesInToOut"),
    Sep,
    Cmd("sequence.matchFrame"),
    Cmd("sequence.reverseMatchFrame"),
    Sep,
    Cmd("sequence.addEdit"),
    Cmd("sequence.addEditAllTracks"),
    Cmd("trim.edit"),
    Tbd("Extend Selected Edit to Playhead"),
    Sep,
    Cmd("sequence.applyVideoTransition"),
    Cmd("sequence.applyAudioTransition"),
    Cmd("trim.applyDefaultTransition"),
    Sep,
    Cmd("sequence.lift"),
    Cmd("sequence.extract"),
    Sep,
    Cmd("view.zoomIn"),
    Cmd("view.zoomOut"),
    Sep,
    Cmd("sequence.closeGap"),
    Sub("Go to Gap", &[Cmd("sequence.goToNextGap"), Cmd("sequence.goToPrevGap"), Cmd("sequence.goToNextGapInTrack"), Cmd("sequence.goToPrevGapInTrack")]),
    Sep,
    Cmd("sequence.snap"),
    Cmd("sequence.linkedSelection"),
    Cmd("sequence.selectionFollowsPlayhead"),
    Cmd("sequence.showThroughEdits"),
    Sep,
    Cmd("sequence.normalizeMixTrack"),
    Sep,
    Cmd("sequence.makeSubsequence"),
    Sep,
    Tbd("Auto Reframe Sequence…"),
    Cmd("sequence.transcribe"),
    Cmd("sequence.simplify"),
    Sep,
    Cmd("sequence.addTracks"),
    Cmd("sequence.deleteTracks"),
    Sep,
    Sub(
        "Captions",
        &[
            Cmd("captions.newTrack"),
            Cmd("captions.add"),
            Sep,
            Cmd("captions.hideAll"),
            Cmd("captions.showAll"),
            Cmd("captions.showActiveOnly"),
            Sep,
            Cmd("captions.next"),
            Cmd("captions.previous"),
        ],
    ),
];

const MARKERS: &[Entry] = &[
    Cmd("markers.markIn"),
    Cmd("markers.markOut"),
    Cmd("markers.markClip"),
    Cmd("markers.markSelection"),
    Sub("Mark Split", &[Cmd("markers.markSplitVideoIn"), Cmd("markers.markSplitVideoOut"), Cmd("markers.markSplitAudioIn"), Cmd("markers.markSplitAudioOut")]),
    Sep,
    Cmd("markers.goToIn"),
    Cmd("markers.goToOut"),
    Sub("Go to Split", &[Cmd("markers.goToSplitVideoIn"), Cmd("markers.goToSplitVideoOut"), Cmd("markers.goToSplitAudioIn"), Cmd("markers.goToSplitAudioOut")]),
    Sep,
    Cmd("markers.clearIn"),
    Cmd("markers.clearOut"),
    Cmd("markers.clearInOut"),
    Sep,
    Cmd("markers.add"),
    Cmd("markers.addRange"),
    Cmd("markers.addRangeInOut"),
    Cmd("markers.goNext"),
    Cmd("markers.goPrev"),
    Sep,
    Cmd("markers.clearCurrent"),
    Cmd("markers.clearAll"),
    Cmd("markers.showAllMarkerColors"),
    Sep,
    Tbd("Edit Marker…"),
    Sep,
    Cmd("markers.addChapter"),
    Cmd("markers.addFlashCue"),
    Sep,
    Cmd("markers.rippleSequenceMarkers"),
    Cmd("markers.copyPasteIncludesSequenceMarkers"),
];

const GRAPHICS_AND_TITLES: &[Entry] = &[
    Cmd("graphics.template.install"),
    Sep,
    Sub(
        "New Layer",
        &[
            Cmd("graphics.newText"),
            Cmd("graphics.newVerticalText"),
            Sep,
            Cmd("graphics.newRectangle"),
            Cmd("graphics.newEllipse"),
            Cmd("graphics.newPolygon"),
            Sep,
            Cmd("graphics.newFromFile"),
        ],
    ),
    Sep,
    Sub(
        "Align to Video Frame",
        &[
            Cmd("graphics.alignFrame.left"),
            Cmd("graphics.alignFrame.hcenter"),
            Cmd("graphics.alignFrame.right"),
            Cmd("graphics.alignFrame.top"),
            Cmd("graphics.alignFrame.vcenter"),
            Cmd("graphics.alignFrame.bottom"),
        ],
    ),
    Sub(
        "Align to Video Frame as Group",
        &[
            Cmd("graphics.alignGroup.left"),
            Cmd("graphics.alignGroup.hcenter"),
            Cmd("graphics.alignGroup.right"),
            Cmd("graphics.alignGroup.top"),
            Cmd("graphics.alignGroup.vcenter"),
            Cmd("graphics.alignGroup.bottom"),
        ],
    ),
    Sub(
        "Align to Selection",
        &[
            Cmd("graphics.alignSelection.left"),
            Cmd("graphics.alignSelection.hcenter"),
            Cmd("graphics.alignSelection.right"),
            Cmd("graphics.alignSelection.top"),
            Cmd("graphics.alignSelection.vcenter"),
            Cmd("graphics.alignSelection.bottom"),
        ],
    ),
    Sub(
        "Distribute",
        &[
            Cmd("graphics.distributeVertically"),
            Cmd("graphics.distributeSpaceVertically"),
            Cmd("graphics.distributeHorizontally"),
            Cmd("graphics.distributeSpaceHorizontally"),
        ],
    ),
    Sub("Arrange", &[Cmd("graphics.bringToFront"), Cmd("graphics.bringForward"), Cmd("graphics.sendBackward"), Cmd("graphics.sendToBack")]),
    Sub(
        "Select",
        &[Cmd("graphics.selectNextGraphic"), Cmd("graphics.selectPreviousGraphic"), Cmd("graphics.selectNextLayer"), Cmd("graphics.selectPreviousLayer")],
    ),
    Sep,
    Cmd("graphics.upgradeToSourceGraphic"),
    Cmd("graphics.upgradeCaption"),
    Cmd("graphics.resetAllParameters"),
    Cmd("graphics.resetDuration"),
    Sep,
    Cmd("graphics.template.export"),
    Sep,
    Cmd("file.replaceFonts"),
];

const VIEW: &[Entry] = &[
    Sub(
        "Playback Resolution",
        &[
            Cmd("view.playbackRes.full"),
            Cmd("view.playbackRes.half"),
            Cmd("view.playbackRes.quarter"),
            Cmd("view.playbackRes.eighth"),
            Cmd("view.playbackRes.sixteenth"),
        ],
    ),
    Sub(
        "Paused Resolution",
        &[Cmd("view.pausedRes.full"), Cmd("view.pausedRes.half"), Cmd("view.pausedRes.quarter"), Cmd("view.pausedRes.eighth"), Cmd("view.pausedRes.sixteenth")],
    ),
    Cmd("view.highQualityPlayback"),
    Sep,
    Sub(
        "Display Mode",
        &[
            Cmd("view.display.composite"),
            Cmd("view.display.alpha"),
            Cmd("view.display.red"),
            Cmd("view.display.green"),
            Cmd("view.display.blue"),
            Cmd("view.display.multicam"),
            Cmd("view.display.audioWaveform"),
            Cmd("view.display.comparison"),
            Cmd("view.display.videoAndWaveform"),
        ],
    ),
    Sub(
        "Magnification",
        &[
            Cmd("view.magnification.fit"),
            Sep,
            Cmd("view.magnification.10"),
            Cmd("view.magnification.25"),
            Cmd("view.magnification.50"),
            Cmd("view.magnification.75"),
            Cmd("view.magnification.100"),
            Cmd("view.magnification.150"),
            Cmd("view.magnification.200"),
            Cmd("view.magnification.400"),
            Cmd("view.magnification.800"),
            Cmd("view.magnification.1600"),
        ],
    ),
    Sep,
    Cmd("view.showRulers"),
    Cmd("view.showGuides"),
    Cmd("view.lockGuides"),
    Cmd("view.addGuide"),
    Cmd("view.clearGuides"),
    Sep,
    Cmd("view.snapInProgramMonitor"),
    Sep,
    Sub("Guide Templates", &[Cmd("view.safeMargins"), Sep, Cmd("view.guideTemplates.save"), Cmd("view.guideTemplates.manage")]),
    Sep,
    Cmd("view.dynamicAudioWaveforms"),
];

const WINDOW: &[Entry] = &[
    Sub(
        "Workspaces",
        &[
            Rest,
            Sep,
            Cmd("window.workspace.reset"),
            Cmd("window.workspace.saveChanges"),
            Cmd("window.workspace.saveAs"),
            Sep,
            Cmd("window.workspace.edit"),
            Sep,
            Tbd("Import Workspace from Projects"),
        ],
    ),
    Sep,
    Tbd("Extensions"),
    Sep,
    Tbd("Maximize Frame"),
    Sep,
    Tbd("Audio Clip Effect Editor"),
    Tbd("Audio Track Effect Editor"),
    Sep,
    Cmd("window.panel.AudioClipMixer"),
    Cmd("window.panel.AudioMeters"),
    Cmd("window.panel.AudioTrackMixer"),
    Cmd("window.panel.EffectControls"),
    Cmd("window.panel.Effects"),
    Cmd("window.panel.EssentialSound"),
    Cmd("window.panel.Events"),
    Cmd("window.panel.EssentialGraphics"),
    Cmd("window.panel.History"),
    Cmd("window.panel.Info"),
    Tbd("Learn"),
    Cmd("window.panel.Libraries"),
    Cmd("window.panel.LumetriColor"),
    Cmd("window.panel.LumetriScopes"),
    Cmd("window.panel.Markers"),
    Cmd("window.panel.MediaBrowser"),
    Cmd("window.panel.Metadata"),
    Cmd("window.panel.Program"),
    Cmd("window.panel.Progress"),
    Cmd("window.panel.Project"),
    Cmd("window.panel.Properties"),
    Tbd("Search"),
    Tbd("Sequence Index"),
    Cmd("window.panel.Source"),
    Cmd("window.panel.Text"),
    Cmd("window.panel.Timecode"),
    Cmd("window.panel.Timeline"),
    Cmd("window.panel.Tools"),
];

const HELP: &[Entry] = &[
    Cmd("help.filmcraftHelp"),
    Tbd("In-App Tutorials…"),
    Tbd("Online Tutorials…"),
    Cmd("help.revealLogFiles"),
    Cmd("help.reportIssue"),
    Sep,
    Cmd("help.systemCompatibilityReport"),
    Tbd("Keyboard…"),
    Sep,
    Tbd("Updates…"),
];

/// The layout of top-level menu `top` (empty for a menu without one: its commands then appear in
/// registration order).
pub fn layout(top: &str) -> &'static [Entry] {
    match top {
        "File" => FILE,
        "Edit" => EDIT,
        "Clip" => CLIP,
        "Sequence" => SEQUENCE,
        "Markers" => MARKERS,
        "Graphics and Titles" => GRAPHICS_AND_TITLES,
        "View" => VIEW,
        "Window" => WINDOW,
        "Help" => HELP,
        _ => &[],
    }
}

fn collect(entries: &'static [Entry], out: &mut HashSet<&'static str>) {
    for e in entries {
        match e {
            Cmd(id) => {
                out.insert(id);
            }
            Sub(_, inner) => collect(inner, out),
            _ => {}
        }
    }
}

/// Every command id a layout names.
fn placed() -> &'static HashSet<&'static str> {
    static P: OnceLock<HashSet<&'static str>> = OnceLock::new();
    P.get_or_init(|| {
        let mut s = HashSet::new();
        for top in crate::menus::MENUS {
            collect(layout(top), &mut s);
        }
        s
    })
}

/// Top-level menu `top` as it is shown, from the registered menu `items`.
pub fn menu<'a>(top: &'a str, items: &'a [MenuItem]) -> Vec<Node<'a>> {
    build(&[top], layout(top), items)
}

fn build<'a>(path: &[&'a str], entries: &'static [Entry], items: &'a [MenuItem]) -> Vec<Node<'a>> {
    let named: Vec<&str> = entries
        .iter()
        .filter_map(|e| match e {
            Sub(n, _) => Some(*n),
            _ => None,
        })
        .collect();
    let mut out = Vec::new();
    let mut rest_placed = false;
    for e in entries {
        match e {
            Sep => out.push(Node::Sep),
            Cmd(id) => out.extend(items.iter().find(|i| i.id == *id).map(Node::Item)),
            Tbd(label) => out.push(Node::Tbd(label)),
            Sub(name, inner) => {
                let mut p = path.to_vec();
                p.push(name);
                let kids = build(&p, inner, items);
                if !kids.is_empty() {
                    out.push(Node::Sub((*name).to_string(), kids));
                }
            }
            Rest => {
                rest_placed = true;
                out.extend(rest(path, &named, items));
            }
        }
    }
    if !rest_placed {
        out.push(Node::Sep);
        out.extend(rest(path, &named, items));
    }
    tidy(out)
}

/// The commands under `path` that no layout names, in registration order; those in a submenu the
/// layout does not have (`named` are the ones it has) make that submenu.
fn rest<'a>(path: &[&'a str], named: &[&str], items: &'a [MenuItem]) -> Vec<Node<'a>> {
    let mut out = Vec::new();
    let mut subs: Vec<&str> = Vec::new();
    for it in items {
        let under = it.path.len() >= path.len() && it.path.iter().zip(path).all(|(a, b)| a == b);
        if !under || placed().contains(it.id.as_str()) {
            continue;
        }
        match it.path.get(path.len()).map(String::as_str) {
            None => out.push(Node::Item(it)),
            Some(sub) if named.contains(&sub) || subs.contains(&sub) => {}
            Some(sub) => {
                subs.push(sub);
                let mut p = path.to_vec();
                p.push(sub);
                out.push(Node::Sub(sub.to_string(), build(&p, &[], items)));
            }
        }
    }
    out
}

/// No separator first, last or twice in a row.
fn tidy(nodes: Vec<Node<'_>>) -> Vec<Node<'_>> {
    let mut out: Vec<Node<'_>> = Vec::new();
    for n in nodes {
        if matches!(n, Node::Sep) && matches!(out.last(), None | Some(Node::Sep)) {
            continue;
        }
        out.push(n);
    }
    while matches!(out.last(), Some(Node::Sep)) {
        out.pop();
    }
    out
}

/// The whole menu bar as JSON, for `ui.menu.tree`: `{label, id?, shortcut?, enabled?, checked?}`
/// items, `{separator: true}`, `{label, tbd: true}` and `{label, items: […]}` submenus.
pub fn tree(items: &[MenuItem]) -> serde_json::Value {
    fn nodes(ns: &[Node<'_>]) -> Vec<serde_json::Value> {
        ns.iter()
            .map(|n| match n {
                Node::Sep => serde_json::json!({"separator": true}),
                Node::Item(it) => serde_json::to_value(it).unwrap_or_default(),
                Node::Tbd(l) => serde_json::json!({"label": l, "tbd": true, "enabled": false}),
                Node::Sub(name, kids) => serde_json::json!({"label": name, "items": nodes(kids)}),
            })
            .collect()
    }
    crate::menus::MENUS.iter().map(|top| serde_json::json!({"label": top, "items": nodes(&menu(top, items))})).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn items() -> Vec<MenuItem> {
        let mut s = filmcraft_engine::Session::default();
        s.execute("file.openDemoProject", serde_json::json!({})).unwrap();
        crate::menus::menu_items(&crate::FilmcraftApp::new(s))
    }

    fn walk<'n, 'a>(nodes: &'n [Node<'a>], path: &str, f: &mut impl FnMut(&str, &'n Node<'a>)) {
        for n in nodes {
            f(path, n);
            if let Node::Sub(name, kids) = n {
                walk(kids, &format!("{path} ▸ {name}"), f);
            }
        }
    }

    #[test]
    fn every_command_the_layout_names_is_registered() {
        // a renamed command would otherwise silently leave its place in the menu
        let items = items();
        let mut missing: Vec<&str> = placed().iter().copied().filter(|id| !items.iter().any(|i| i.id == *id)).collect();
        missing.sort_unstable();
        assert!(missing.is_empty(), "the layout names commands that are not in any menu: {missing:?}");
    }

    #[test]
    fn every_menu_command_is_shown_exactly_once() {
        let items = items();
        let mut seen: Vec<&str> = Vec::new();
        for top in crate::menus::MENUS {
            let m = menu(top, &items);
            walk(&m, top, &mut |_, n| {
                if let Node::Item(it) = n {
                    let it: &MenuItem = it;
                    seen.push(it.id.as_str());
                }
            });
        }
        for it in &items {
            let n = seen.iter().filter(|id| **id == it.id).count();
            assert_eq!(n, 1, "{} ({}) is in the menus {n} times", it.id, it.path.join(" ▸ "));
        }
        assert_eq!(seen.len(), items.len());
    }

    #[test]
    fn separators_only_between_items_and_no_empty_submenus() {
        let items = items();
        for top in crate::menus::MENUS {
            let m = menu(top, &items);
            let check = |path: &str, level: &[Node<'_>]| {
                assert!(!level.is_empty(), "{path} is empty");
                assert!(!matches!(level.first(), Some(Node::Sep)) && !matches!(level.last(), Some(Node::Sep)), "{path} starts or ends with a separator");
                assert!(!level.windows(2).any(|w| matches!(w, [Node::Sep, Node::Sep])), "{path} has two separators in a row");
            };
            check(top, &m);
            walk(&m, top, &mut |path, n| {
                if let Node::Sub(name, kids) = n {
                    check(&format!("{path} ▸ {name}"), kids);
                }
            });
        }
    }

    #[test]
    fn a_tbd_item_is_not_something_filmcraft_already_has() {
        let norm = |s: &str| s.replace('…', "").replace("...", "").trim().to_lowercase();
        let items = items();
        for top in crate::menus::MENUS {
            let m = menu(top, &items);
            let mut tbd: Vec<(String, &str)> = Vec::new();
            walk(&m, top, &mut |path, n| {
                if let Node::Tbd(l) = n {
                    tbd.push((path.to_string(), l));
                }
            });
            for (path, label) in tbd {
                let twin = items.iter().find(|it| it.path.join(" ▸ ") == path && norm(&it.label) == norm(label));
                assert!(twin.is_none(), "{path} ▸ {label} is marked TBD but is command {}", twin.map_or("", |t| t.id.as_str()));
            }
        }
    }

    #[test]
    fn menus_follow_premiere() {
        let items = items();
        let line = |n: &Node<'_>| match n {
            Node::Sep => "---".to_string(),
            Node::Item(it) => it.label.clone(),
            Node::Tbd(l) => format!("{l} {TBD}"),
            Node::Sub(name, _) => format!("{name} >"),
        };
        let seq: Vec<String> = menu("Sequence", &items).iter().map(line).collect();
        assert_eq!(seq[..4], ["Sequence Settings…", "---", "Render Effects In to Out", "Render In to Out"], "{seq:?}");
        let markers: Vec<String> = menu("Markers", &items).iter().map(line).collect();
        assert_eq!(markers[..6], ["Mark In", "Mark Out", "Mark Clip", "Mark Selection", "Mark Split >", "---"], "{markers:?}");
        let file: Vec<String> = menu("File", &items).iter().map(line).collect();
        assert!(file.contains(&format!("Open Recent {TBD}")), "{file:?}");
        // FilmCraft's own commands follow Premiere's, after a separator
        let edit: Vec<String> = menu("Edit", &items).iter().map(line).collect();
        let at = |l: &str| edit.iter().position(|x| x == l).unwrap_or_else(|| panic!("no {l} in {edit:?}"));
        assert!(at("Keyboard Shortcuts…") > at("Undo") && at("Preferences >") > at("Paste"));
    }
}
