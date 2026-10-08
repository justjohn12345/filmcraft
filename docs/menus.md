# Menus

FilmCraft draws its own menus on every platform: the menu bar in the window's header, the panel
menus (the "≡" beside a panel's active tab, or a right-click on a tab) and the context menus.

## Menu bar

The nine menus (File, Edit, Clip, Sequence, Markers, Graphics and Titles, View, Window, Help)
follow Premiere Pro's: the same items in the same order with the same separator lines (observed in
Premiere Pro 2026 on macOS). The layout is one table, `crates/ui-egui/src/menu_layout.rs`:

| Entry | Meaning |
|---|---|
| `Cmd("id")` | a registered command; its label, shortcut, enabled and checked state come from the registry |
| `Tbd("Label")` | an item Premiere has and FilmCraft does not yet: shown greyed out as `Label (TBD)` |
| `Sep` | a separator line |
| `Sub("Name", …)` | a submenu |
| `Rest` | where the menu's commands that the table does not name go |

A command that is registered with a menu path but not named in the table is not lost: it goes to
the end of its menu (or submenu) after a separator, in registration order. That is where FilmCraft's
own commands are (Edit ▸ Keyboard Shortcuts…, Language and Preferences; the Transcript, Proxy and
Appearance submenus; Help's links). To give a new command its place, add a `Cmd` line; a test fails
when the table names a command that does not exist, and when a command is shown twice or not at all.

Left out on purpose:

- items that only make sense with another vendor's products or services (stock libraries,
  generative services, review services, companion apps, extension stores);
- Productions and Team Projects, which are out of scope (see [ROADMAP.md](../ROADMAP.md));
- the items macOS adds to every Edit menu (Writing Tools, AutoFill, Start Dictation, Emoji & Symbols);
- account items (sign in / out, manage account);
- Premiere's application menu. About is in Help, and Keyboard Shortcuts… and Preferences are at the
  end of Edit, where Premiere has them on Windows.

Differences from Premiere: Sequence ▸ Zoom In / Zoom Out are followed by FilmCraft's Zoom to
Sequence under View; Window lists one item per monitor and Timeline panel where Premiere has a
submenu of the open sequences; the workspaces listed are FilmCraft's.

Checked items: the open panels in Window, Snap in Timeline, Linked Selection, the View toggles and
the interface language.

## Panel menus

Every panel menu starts as Premiere's do: Close Panel, Undock Panel, Close Other Panels in Group,
(Timeline: Close Other Timeline Panels,) Panel Group Settings ▸. Below a separator come the panel's
own entries in Premiere's order, from `panel_layout` in `crates/ui-egui/src/panels/mod.rs` (Program,
Source, Effect Controls, Effects, Timeline) or drawn by the panel (Project, Media Browser). Entries
FilmCraft does not have yet are `(TBD)` rows here too. Panel Group Settings ▸ Maximize Panel Group
fills the window with the group; Restore Workspace (FilmCraft's, in the same submenu) brings the
layout back. The Timeline menu ends with FilmCraft's Video Thumbnails and Audio Waveforms toggles.

Not yet compared with Premiere: the menus of the panels that were not open in the workspace used
for the comparison (Audio Clip Mixer, Audio Track Mixer, Metadata, Markers, History, Info,
Libraries, Lumetri Color, Lumetri Scopes, Essential Sound, Text, Events, Progress, Timecode).

## Right-click and pop-up menus

Every right-click menu and every pop-up menu (wrench buttons, drop-down lists of commands) uses
the same rows and look as the menu bar: `menus::context_menu`, `menus::entry`, `menus::tbd`,
`menus::separator`. The long ones (Timeline clip, Project item, the monitors' picture and Settings
menus) scroll when they are taller than the window.

These hold Premiere Pro's rows in Premiere's order, with `(TBD)` rows for what FilmCraft lacks
(read from Premiere Pro 2026 on macOS; FilmCraft's own rows follow at the end after a separator):

| Element | Menu |
|---|---|
| Effect Controls: a keyframe | Cut, Copy, Paste, Clear, Select All, the interpolations (the keyframe's is checked), Ease In, Ease Out |
| Effect Controls: an effect's header, a property's name, the empty keyframe lane, the time ruler | rename / preset / clipboard rows; the ruler has the In / Out and marker rows |
| Timeline: a video clip, an audio clip | the clip menu (they differ in Show Clip Keyframes) |
| Timeline: the time ruler | In / Out and marker rows |
| Timeline: a track header | Rename, Add Track, Delete Track, Add Tracks…, Delete Tracks…, caption and targeted-track rows |
| Timeline: the timecode, the Display Settings wrench | time display formats; display toggles |
| Program and Source monitor: the picture, the timecode, the time ruler, the Settings wrench | marks, markers, resolution, display mode, magnification, overlays, multi-camera |
| Project panel: an item, a column header | the item menu (clip and sequence variants); Metadata Display… |
| Audio Meters | Reset Indicators, the ranges and peak modes |

Right-clicking empty track space or a transition opens no menu.

Converted to the common look only, with their rows unchanged, because Premiere's versions were not
compared: Project panel bins, empty space, Freeform cards and canvas, New Item and Sort Icons;
Media Browser tree, entries, columns and File Types; Effects panel presets; Lumetri Scopes;
Reference Monitor; Timecode panel; the Tools flyouts; the mixers' slot and value pop-ups; the Text
panel's caption track list; the header's Workspaces list and the tab overflow list; mask rows.
The Source monitor's menus are the Program monitor's without the rows that need a sequence;
Premiere's own Source menus were not compared.

## Look

`theme::menu_style` styles every menu: rows start after a gutter that holds the checkmark of a
checked item, shortcuts are at the right in the platform's notation, separators are inset lines.
On macOS the menu is a rounded panel and the row under the pointer is a rounded bar in the accent
colour; on Windows and Linux the panel is squarer and the row is a quiet grey.

## Automation

| Id | Element |
|---|---|
| `menubar.<Menu>` | a menu bar button (`menubar.GraphicsandTitles`) |
| `menu.<command id>` | an item of an open menu or submenu |
| `menu.submenu.<Name>` | a submenu's row (hover it to open the submenu) |
| `panel.menu.<Panel>` | a panel's "≡" |
| `panel.menu.<Panel>.close`, `.groupSettings`, `.maximize`, `.restoreWorkspace` | the common entries |
| `panel.menu.<Panel>.<command id>` | a panel's own entry that runs a command |
| `panel.menu.Timeline.closeOthers`, `.thumbnails`, `.waveforms` | Timeline entries |
| `project.menu.*`, `mediaBrowser.menu.*` | the Project and Media Browser panels' entries |
| `effectControls.<effect>.<param>.keyframe.<time>.<row>`, `effectControls.ruler.<command id>` | Effect Controls keyframe and ruler menu rows |
| `timeline.clipMenu.<command id>`, `timeline.rulerMenu.<command id>`, `timeline.trackMenu.<row>`, `timeline.track.<V1…>.header` | Timeline menus and the header to right-click |
| `<monitor>.picture.menu.<key>`, `<monitor>.scrubBar.menu.<key>`, `<monitor>.settings.<key>` | monitor picture, time ruler and Settings menu rows |
| `project.itemMenu.<row>`, `audioMeters.menu.<row>` | Project item and Audio Meters menu rows |

A row scrolled out of a tall menu is not an on-screen element until the menu is scrolled (`ui.scroll`).

`ui.menu.tree` returns the menu bar as it is shown; `ui.menu.invoke` runs any item by command id
without opening a menu.
