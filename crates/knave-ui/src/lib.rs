//! Renderer-independent primitives for the Knave shell UI.

use std::sync::Arc;

use knave_desktop_api::{DesktopSnapshot, WindowId, WindowSummary, WorkspaceId};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub alpha: u8,
}

impl Color {
    pub const BACKGROUND: Self = Self::rgba(20, 20, 20, 255);
    pub const SURFACE: Self = Self::rgba(34, 34, 34, 255);
    pub const SELECTED: Self = Self::rgba(64, 64, 64, 255);
    pub const ACCENT: Self = Self::rgba(112, 112, 112, 255);
    pub const TEXT: Self = Self::rgba(238, 238, 238, 255);
    pub const MUTED_TEXT: Self = Self::rgba(166, 166, 166, 255);

    pub const fn rgba(red: u8, green: u8, blue: u8, alpha: u8) -> Self {
        Self {
            red,
            green,
            blue,
            alpha,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiImage {
    width: u32,
    height: u32,
    pixels: Arc<[u8]>,
}

impl UiImage {
    pub fn from_rgba(width: u32, height: u32, pixels: Vec<u8>) -> Option<Self> {
        let expected = (width as usize)
            .checked_mul(height as usize)?
            .checked_mul(4)?;
        if pixels.len() != expected || width == 0 || height == 0 {
            return None;
        }
        Some(Self {
            width,
            height,
            pixels: Arc::from(pixels.into_boxed_slice()),
        })
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn pixels(&self) -> &[u8] {
        &self.pixels
    }

    pub fn cache_key(&self) -> usize {
        Arc::as_ptr(&self.pixels) as *const u8 as usize
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkspacePreviewImage {
    pub workspace: WorkspaceId,
    pub image: UiImage,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplicationSummary {
    pub id: String,
    pub name: String,
    pub generic_name: String,
    pub keywords: Vec<String>,
    pub window_app_ids: Vec<String>,
    pub exec_argv: Vec<String>,
    pub icon: Option<UiImage>,
}

#[derive(Clone, Copy, Debug)]
pub struct OverviewSceneInput<'a> {
    pub snapshot: Option<&'a DesktopSnapshot>,
    pub query: &'a str,
    pub selected: usize,
    pub previews: &'a [WorkspacePreviewImage],
    pub applications: Option<&'a [ApplicationSummary]>,
    pub clock: &'a str,
    pub logo: Option<&'a UiImage>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NodeId(pub u64);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiAction {
    CloseOverview,
    OpenOverview,
    FocusSearch,
    FocusWorkspace(WorkspaceId),
    FocusWindow(WindowId),
    RestoreWindow(WindowId),
    LaunchApplication(usize),
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct HitTarget {
    bounds: Rect,
    action: UiAction,
}

#[derive(Clone, Debug, PartialEq)]
pub enum UiNode {
    Panel {
        id: NodeId,
        bounds: Rect,
        color: Color,
    },
    Label {
        id: NodeId,
        bounds: Rect,
        color: Color,
        text: String,
    },
    Image {
        id: NodeId,
        bounds: Rect,
        image: UiImage,
    },
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct UiScene {
    revision: u64,
    nodes: Vec<UiNode>,
    targets: Vec<HitTarget>,
    search_actions: Vec<UiAction>,
}

const MAX_OVERVIEW_WINDOWS: usize = 12;
const MAX_MINIMIZED_WINDOWS: usize = 24;
const MAX_OVERVIEW_WORKSPACES: usize = 10;
const MAX_SEARCH_RESULTS: usize = 12;
pub const MAX_SEARCH_QUERY: usize = 64;
const OVERVIEW_COLUMNS: usize = 4;
const ICON_SIZE: f32 = 28.0;
const OVERVIEW_STATUS_NODE_ID_OFFSET: u64 = 10_000;

fn bounded_text(value: &str, max: usize) -> String {
    value.chars().take(max).collect()
}

fn normalized_app_id(value: &str) -> String {
    value.trim().trim_end_matches(".desktop").to_lowercase()
}

fn window_label(window: &WindowSummary) -> String {
    let label = if window.title.is_empty() {
        window.app_id.as_str()
    } else {
        window.title.as_str()
    };
    bounded_text(label, 48)
}

fn application_for_window<'a>(
    applications: &'a [ApplicationSummary],
    window: &WindowSummary,
) -> Option<&'a ApplicationSummary> {
    let id = normalized_app_id(&window.app_id);
    applications.iter().find(|application| {
        application
            .window_app_ids
            .iter()
            .any(|alias| normalized_app_id(alias) == id)
    })
}

#[derive(Clone, Debug)]
struct SearchResult {
    title: String,
    icon: Option<UiImage>,
    action: UiAction,
}

fn application_matches(application: &ApplicationSummary, query: &str) -> bool {
    let needle = query.trim().to_lowercase();
    if needle.is_empty() {
        return false;
    }
    std::iter::once(application.id.as_str())
        .chain(std::iter::once(application.name.as_str()))
        .chain(std::iter::once(application.generic_name.as_str()))
        .chain(application.keywords.iter().map(String::as_str))
        .any(|value| value.to_lowercase().contains(&needle))
}

fn search_results(applications: &[ApplicationSummary], query: &str) -> Vec<SearchResult> {
    if query.trim().is_empty() {
        return Vec::new();
    }

    applications
        .iter()
        .enumerate()
        .filter(|(_, application)| application_matches(application, query))
        .take(MAX_SEARCH_RESULTS)
        .map(|(index, application)| SearchResult {
            title: bounded_text(&application.name, 48),
            icon: application.icon.clone(),
            action: UiAction::LaunchApplication(index),
        })
        .collect()
}

impl UiScene {
    pub fn new(revision: u64) -> Self {
        Self {
            revision,
            nodes: Vec::new(),
            targets: Vec::new(),
            search_actions: Vec::new(),
        }
    }

    pub fn revision(&self) -> u64 {
        self.revision
    }

    pub fn nodes(&self) -> &[UiNode] {
        &self.nodes
    }

    pub fn push(&mut self, node: UiNode) {
        self.nodes.push(node);
    }

    pub fn hit_test(&self, x: f32, y: f32) -> Option<UiAction> {
        self.targets
            .iter()
            .rev()
            .find(|target| {
                x >= target.bounds.x
                    && x < target.bounds.x + target.bounds.width
                    && y >= target.bounds.y
                    && y < target.bounds.y + target.bounds.height
            })
            .map(|target| target.action)
    }

    pub fn search_action(&self, index: usize) -> Option<UiAction> {
        self.search_actions.get(index).copied()
    }

    pub fn search_result_count(&self) -> usize {
        self.search_actions.len()
    }

    fn target(&mut self, bounds: Rect, action: UiAction) {
        self.targets.push(HitTarget { bounds, action });
    }

    fn add_logo(&mut self, bounds: Rect, logo: Option<&UiImage>, action: UiAction, id: u64) {
        if let Some(image) = logo {
            self.push(UiNode::Image {
                id: NodeId(id),
                bounds,
                image: image.clone(),
            });
        } else {
            self.push(UiNode::Label {
                id: NodeId(id),
                bounds,
                color: Color::TEXT,
                text: "K".into(),
            });
        }
        self.target(bounds, action);
    }

    fn status_strip(
        revision: u64,
        width: f32,
        height: f32,
        snapshot: Option<&DesktopSnapshot>,
        clock: &str,
        logo: Option<&UiImage>,
        logo_action: UiAction,
    ) -> Self {
        let mut scene = Self::new(revision);
        scene.push(UiNode::Panel {
            id: NodeId(1),
            bounds: Rect::new(0.0, 0.0, width, height),
            color: Color::BACKGROUND,
        });
        let logo_bounds = Rect::new(8.0, ((height - 24.0) / 2.0).max(0.0), 24.0, 24.0);
        scene.add_logo(logo_bounds, logo, logo_action, 2);

        if let Some(snapshot) = snapshot {
            for (index, workspace) in snapshot
                .workspaces
                .iter()
                .take(MAX_OVERVIEW_WORKSPACES)
                .enumerate()
            {
                let bounds = Rect::new(44.0 + index as f32 * 36.0, 4.0, 32.0, height - 8.0);
                if workspace.active {
                    scene.push(UiNode::Panel {
                        id: NodeId(10 + index as u64),
                        bounds,
                        color: Color::SELECTED,
                    });
                }
                scene.push(UiNode::Label {
                    id: NodeId(30 + index as u64),
                    bounds,
                    color: if workspace.active {
                        Color::TEXT
                    } else {
                        Color::MUTED_TEXT
                    },
                    text: workspace.workspace.0.to_string(),
                });
                scene.target(bounds, UiAction::FocusWorkspace(workspace.workspace));
            }
        }

        if !clock.is_empty() {
            let clock_width = clock.chars().count() as f32 * 12.0;
            scene.push(UiNode::Label {
                id: NodeId(60),
                bounds: Rect::new(
                    ((width - clock_width) / 2.0).max(0.0),
                    (height - 22.0) / 2.0,
                    clock_width.min(width),
                    22.0,
                ),
                color: Color::TEXT,
                text: clock.to_owned(),
            });
        }
        scene
    }

    pub fn bar(revision: u64, width: f32, height: f32) -> Self {
        Self::status_strip(
            revision,
            width,
            height,
            None,
            "",
            None,
            UiAction::OpenOverview,
        )
    }

    pub fn bar_with_snapshot(
        revision: u64,
        width: f32,
        height: f32,
        snapshot: Option<&DesktopSnapshot>,
    ) -> Self {
        Self::bar_with_status(revision, width, height, snapshot, "", None)
    }

    pub fn bar_with_status(
        revision: u64,
        width: f32,
        height: f32,
        snapshot: Option<&DesktopSnapshot>,
        clock: &str,
        logo: Option<&UiImage>,
    ) -> Self {
        Self::status_strip(
            revision,
            width,
            height,
            snapshot,
            clock,
            logo,
            UiAction::OpenOverview,
        )
    }

    pub fn overview(revision: u64, width: f32, height: f32) -> Self {
        Self::overview_with_data_and_search(
            revision,
            width,
            height,
            OverviewSceneInput {
                snapshot: None,
                query: "",
                selected: 0,
                previews: &[],
                applications: None,
                clock: "",
                logo: None,
            },
        )
    }

    fn overview_background(revision: u64, width: f32, height: f32) -> Self {
        let mut scene = Self::new(revision);
        scene.push(UiNode::Panel {
            id: NodeId(1),
            bounds: Rect::new(0.0, 0.0, width, height),
            color: Color::BACKGROUND,
        });
        scene.target(Rect::new(0.0, 0.0, width, height), UiAction::CloseOverview);
        scene
    }

    fn append_status_strip(
        &mut self,
        width: f32,
        snapshot: Option<&DesktopSnapshot>,
        clock: &str,
        logo: Option<&UiImage>,
    ) {
        let mut strip = Self::status_strip(
            self.revision,
            width,
            36.0,
            snapshot,
            clock,
            logo,
            UiAction::CloseOverview,
        );
        for node in &mut strip.nodes {
            match node {
                UiNode::Panel { id, .. } | UiNode::Label { id, .. } | UiNode::Image { id, .. } => {
                    id.0 += OVERVIEW_STATUS_NODE_ID_OFFSET;
                }
            }
        }
        self.nodes.append(&mut strip.nodes);
        self.targets.append(&mut strip.targets);
    }

    fn add_search_field(&mut self, width: f32, query: &str) {
        let field_width = width.clamp(1.0, 560.0);
        let bounds = Rect::new((width - field_width) / 2.0, 60.0, field_width, 40.0);
        self.push(UiNode::Panel {
            id: NodeId(10),
            bounds,
            color: Color::SURFACE,
        });
        self.push(UiNode::Label {
            id: NodeId(11),
            bounds: Rect::new(bounds.x + 14.0, bounds.y + 4.0, bounds.width - 28.0, 32.0),
            color: if query.is_empty() {
                Color::MUTED_TEXT
            } else {
                Color::TEXT
            },
            text: if query.is_empty() {
                "Search apps".into()
            } else {
                bounded_text(query, MAX_SEARCH_QUERY)
            },
        });
        self.target(bounds, UiAction::FocusSearch);
    }

    fn add_search_results(
        &mut self,
        applications: Option<&[ApplicationSummary]>,
        query: &str,
        selected: usize,
        width: f32,
    ) {
        let results = search_results(applications.unwrap_or(&[]), query);
        let panel_width = width.clamp(1.0, 560.0);
        let panel_x = ((width - panel_width) / 2.0).max(0.0);
        let row_height = 52.0;
        let panel_height = (results.len().max(1) as f32 * row_height + 8.0).min(640.0);
        let panel = Rect::new(panel_x, 108.0, panel_width, panel_height);
        self.push(UiNode::Panel {
            id: NodeId(3_000),
            bounds: panel,
            color: Color::SURFACE,
        });

        if results.is_empty() {
            self.push(UiNode::Label {
                id: NodeId(3_001),
                bounds: Rect::new(panel.x + 12.0, panel.y + 8.0, panel.width - 24.0, 32.0),
                color: Color::MUTED_TEXT,
                text: if applications.is_some() {
                    "No apps found".into()
                } else {
                    "Loading apps".into()
                },
            });
            return;
        }

        let selected = selected.min(results.len() - 1);
        for (index, result) in results.into_iter().enumerate() {
            let bounds = Rect::new(
                panel.x + 4.0,
                panel.y + 4.0 + index as f32 * row_height,
                panel.width - 8.0,
                row_height - 4.0,
            );
            if index == selected {
                self.push(UiNode::Panel {
                    id: NodeId(3_100 + index as u64),
                    bounds,
                    color: Color::SELECTED,
                });
            }
            if let Some(icon) = result.icon {
                self.push(UiNode::Image {
                    id: NodeId(3_200 + index as u64),
                    bounds: Rect::new(bounds.x + 10.0, bounds.y + 6.0, 36.0, 36.0),
                    image: icon,
                });
            } else {
                self.push(UiNode::Panel {
                    id: NodeId(3_200 + index as u64),
                    bounds: Rect::new(bounds.x + 10.0, bounds.y + 6.0, 36.0, 36.0),
                    color: Color::SELECTED,
                });
            }
            self.push(UiNode::Label {
                id: NodeId(3_300 + index as u64),
                bounds: Rect::new(bounds.x + 58.0, bounds.y + 8.0, bounds.width - 70.0, 32.0),
                color: Color::TEXT,
                text: result.title,
            });
            self.search_actions.push(result.action);
            self.target(bounds, result.action);
        }
    }

    fn add_window_card(
        &mut self,
        index: usize,
        bounds: Rect,
        window: &WindowSummary,
        applications: &[ApplicationSummary],
        minimized: bool,
    ) {
        self.push(UiNode::Panel {
            id: NodeId(1_000 + index as u64),
            bounds,
            color: if window.focused {
                Color::SELECTED
            } else {
                Color::SURFACE
            },
        });
        let icon_bounds = Rect::new(
            bounds.x + 10.0,
            bounds.y + (bounds.height - ICON_SIZE) / 2.0,
            ICON_SIZE,
            ICON_SIZE,
        );
        if let Some(icon) = application_for_window(applications, window)
            .and_then(|application| application.icon.as_ref())
        {
            self.push(UiNode::Image {
                id: NodeId(2_000 + index as u64),
                bounds: icon_bounds,
                image: icon.clone(),
            });
        } else {
            self.push(UiNode::Panel {
                id: NodeId(2_000 + index as u64),
                bounds: icon_bounds,
                color: Color::SELECTED,
            });
            let initial = window_label(window)
                .chars()
                .next()
                .unwrap_or('?')
                .to_uppercase()
                .to_string();
            self.push(UiNode::Label {
                id: NodeId(2_100 + index as u64),
                bounds: icon_bounds,
                color: Color::TEXT,
                text: initial,
            });
        }
        self.push(UiNode::Label {
            id: NodeId(2_200 + index as u64),
            bounds: Rect::new(
                bounds.x + 48.0,
                bounds.y + 8.0,
                (bounds.width - 56.0).max(1.0),
                bounds.height - 16.0,
            ),
            color: Color::TEXT,
            text: window_label(window),
        });
        self.target(
            bounds,
            if minimized {
                UiAction::RestoreWindow(window.id)
            } else {
                UiAction::FocusWindow(window.id)
            },
        );
    }

    pub fn overview_with_data_and_search(
        revision: u64,
        width: f32,
        height: f32,
        input: OverviewSceneInput<'_>,
    ) -> Self {
        let OverviewSceneInput {
            snapshot,
            query,
            selected,
            previews,
            applications,
            clock,
            logo,
        } = input;
        let mut scene = Self::overview_background(revision, width, height);
        scene.append_status_strip(width, snapshot, clock, logo);
        scene.add_search_field(width, query);

        if !query.is_empty() {
            scene.add_search_results(applications, query, selected, width);
            return scene;
        }

        let Some(snapshot) = snapshot else {
            return scene;
        };
        let Some(active_workspace) = snapshot
            .workspaces
            .iter()
            .find(|workspace| workspace.active)
        else {
            return scene;
        };

        let gap = 12.0;
        let workspace_count = snapshot.workspaces.len().min(MAX_OVERVIEW_WORKSPACES);
        let mut window_top = 132.0;
        if workspace_count > 0 {
            let preview_width = ((width - 64.0 - gap * (workspace_count as f32 - 1.0))
                / workspace_count as f32)
                .max(1.0);
            let preview_height = (preview_width * 9.0 / 16.0).clamp(96.0, 200.0);
            for (index, workspace) in snapshot.workspaces.iter().take(workspace_count).enumerate() {
                let bounds = Rect::new(
                    32.0 + index as f32 * (preview_width + gap),
                    132.0,
                    preview_width,
                    preview_height,
                );
                scene.push(UiNode::Panel {
                    id: NodeId(500 + index as u64),
                    bounds,
                    color: if workspace.active {
                        Color::ACCENT
                    } else {
                        Color::SURFACE
                    },
                });
                if let Some(preview) = previews
                    .iter()
                    .find(|preview| preview.workspace == workspace.workspace)
                {
                    scene.push(UiNode::Image {
                        id: NodeId(600 + index as u64),
                        bounds: Rect::new(
                            bounds.x + 2.0,
                            bounds.y + 2.0,
                            (bounds.width - 4.0).max(1.0),
                            (bounds.height - 28.0).max(1.0),
                        ),
                        image: preview.image.clone(),
                    });
                }
                scene.push(UiNode::Label {
                    id: NodeId(700 + index as u64),
                    bounds: Rect::new(
                        bounds.x + 8.0,
                        bounds.y + bounds.height - 24.0,
                        (bounds.width - 16.0).max(1.0),
                        20.0,
                    ),
                    color: Color::TEXT,
                    text: format!("Workspace {}", workspace.workspace.0),
                });
                scene.target(bounds, UiAction::FocusWorkspace(workspace.workspace));
            }
            window_top = 132.0 + (preview_width * 9.0 / 16.0).clamp(96.0, 200.0) + 28.0;
        }

        let visible_windows = snapshot
            .windows
            .iter()
            .filter(|window| {
                window.workspace == active_workspace.workspace
                    && !window.minimized
                    && !window.floating
            })
            .take(MAX_OVERVIEW_WINDOWS)
            .collect::<Vec<_>>();
        if !visible_windows.is_empty() {
            scene.push(UiNode::Label {
                id: NodeId(800),
                bounds: Rect::new(32.0, window_top, width - 64.0, 20.0),
                color: Color::MUTED_TEXT,
                text: "Windows".into(),
            });
            let card_width = ((width - 64.0 - gap * (OVERVIEW_COLUMNS as f32 - 1.0))
                / OVERVIEW_COLUMNS as f32)
                .max(1.0);
            let card_height = 60.0;
            let card_top = window_top + 26.0;
            for (index, window) in visible_windows.iter().enumerate() {
                let column = index % OVERVIEW_COLUMNS;
                let row = index / OVERVIEW_COLUMNS;
                let bounds = Rect::new(
                    32.0 + column as f32 * (card_width + gap),
                    card_top + row as f32 * (card_height + gap),
                    card_width,
                    card_height,
                );
                scene.add_window_card(index, bounds, window, applications.unwrap_or(&[]), false);
            }
            let rows = visible_windows.len().div_ceil(OVERVIEW_COLUMNS);
            window_top = card_top + rows as f32 * (card_height + gap);
        }

        let minimized = snapshot
            .windows
            .iter()
            .filter(|window| window.workspace == active_workspace.workspace && window.minimized)
            .take(MAX_MINIMIZED_WINDOWS)
            .collect::<Vec<_>>();
        if !minimized.is_empty() {
            scene.push(UiNode::Label {
                id: NodeId(900),
                bounds: Rect::new(32.0, window_top, width - 64.0, 20.0),
                color: Color::MUTED_TEXT,
                text: "Minimized".into(),
            });
            let card_width = ((width - 64.0 - gap * (OVERVIEW_COLUMNS as f32 - 1.0))
                / OVERVIEW_COLUMNS as f32)
                .max(1.0);
            let card_height = 56.0;
            let card_top = window_top + 26.0;
            for (index, window) in minimized.iter().enumerate() {
                let column = index % OVERVIEW_COLUMNS;
                let row = index / OVERVIEW_COLUMNS;
                let bounds = Rect::new(
                    32.0 + column as f32 * (card_width + gap),
                    card_top + row as f32 * (card_height + gap),
                    card_width,
                    card_height,
                );
                scene.add_window_card(
                    MAX_OVERVIEW_WINDOWS + index,
                    bounds,
                    window,
                    applications.unwrap_or(&[]),
                    true,
                );
            }
        }
        scene
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use knave_desktop_api::{DesktopSnapshot, WorkspaceSummary};

    fn snapshot() -> DesktopSnapshot {
        DesktopSnapshot {
            generation: 4,
            workspaces: vec![WorkspaceSummary {
                workspace: WorkspaceId(2),
                active: true,
                window_count: 2,
                visible_window_count: 1,
            }],
            windows: vec![
                WindowSummary {
                    id: WindowId(41),
                    title: "Terminal".into(),
                    app_id: "foot".into(),
                    workspace: WorkspaceId(2),
                    focused: true,
                    minimized: false,
                    floating: false,
                    fullscreen: false,
                },
                WindowSummary {
                    id: WindowId(42),
                    title: "Editor".into(),
                    app_id: "code".into(),
                    workspace: WorkspaceId(2),
                    focused: false,
                    minimized: true,
                    floating: false,
                    fullscreen: false,
                },
            ],
        }
    }

    fn applications(icon: Option<UiImage>) -> Vec<ApplicationSummary> {
        vec![ApplicationSummary {
            id: "org.example.Editor.desktop".into(),
            name: "Example Editor".into(),
            generic_name: "Text Editor".into(),
            keywords: vec!["code".into()],
            window_app_ids: vec!["org.example.Editor".into(), "code".into()],
            exec_argv: vec!["editor".into()],
            icon,
        }]
    }

    fn overview_scene(
        revision: u64,
        snapshot: Option<&DesktopSnapshot>,
        query: &str,
        applications: Option<&[ApplicationSummary]>,
    ) -> UiScene {
        UiScene::overview_with_data_and_search(
            revision,
            1920.0,
            1080.0,
            OverviewSceneInput {
                snapshot,
                query,
                selected: 0,
                previews: &[],
                applications,
                clock: "",
                logo: None,
            },
        )
    }

    #[test]
    fn bar_scene_has_a_left_logo_button_and_center_clock() {
        let logo = UiImage::from_rgba(24, 24, vec![255; 24 * 24 * 4]).unwrap();
        let scene = UiScene::bar_with_status(
            7,
            1920.0,
            36.0,
            Some(&snapshot()),
            "Tue Apr 22 10:24 AM",
            Some(&logo),
        );
        assert_eq!(scene.revision(), 7);
        assert_eq!(scene.hit_test(10.0, 10.0), Some(UiAction::OpenOverview));
        assert!(scene.nodes().iter().any(|node| matches!(node,
            UiNode::Label { bounds, text, .. }
                if text == "Tue Apr 22 10:24 AM" && (bounds.x + bounds.width / 2.0 - 960.0).abs() < 1.0
        )));
        assert!(scene.nodes().iter().any(|node| matches!(node,
            UiNode::Label { text, .. } if text == "2"
        )));
    }

    #[test]
    fn overview_has_workspaces_windows_and_minimized_restore_targets() {
        let scene = overview_scene(1, Some(&snapshot()), "", Some(&applications(None)));
        assert_eq!(scene.hit_test(10.0, 10.0), Some(UiAction::CloseOverview));
        assert_eq!(
            scene.hit_test(50.0, 10.0),
            Some(UiAction::FocusWorkspace(WorkspaceId(2)))
        );
        assert_eq!(
            scene.hit_test(40.0, 400.0),
            Some(UiAction::FocusWindow(WindowId(41)))
        );
        assert_eq!(
            scene.hit_test(40.0, 500.0),
            Some(UiAction::RestoreWindow(WindowId(42)))
        );
        assert!(scene.nodes().iter().any(|node| matches!(node,
            UiNode::Label { text, .. } if text == "Windows"
        )));
        assert!(scene.nodes().iter().any(|node| matches!(node,
            UiNode::Label { text, .. } if text == "Minimized"
        )));
    }

    #[test]
    fn app_search_never_returns_windows_or_workspaces() {
        let snapshot = snapshot();
        let apps = applications(None);
        let scene = overview_scene(1, Some(&snapshot), "editor", Some(&apps));
        assert_eq!(scene.search_result_count(), 1);
        assert_eq!(scene.search_action(0), Some(UiAction::LaunchApplication(0)));
        assert!(scene.nodes().iter().any(|node| matches!(node,
            UiNode::Label { text, .. } if text == "Example Editor"
        )));
        assert!(!scene.nodes().iter().any(|node| matches!(node,
            UiNode::Label { text, .. } if text == "Windows" || text == "Minimized" || text == "Workspace 2"
        )));
        let no_workspace_result = overview_scene(2, Some(&snapshot), "workspace 2", Some(&apps));
        assert_eq!(no_workspace_result.search_result_count(), 0);
    }

    #[test]
    fn search_and_minimized_cards_render_actual_icons() {
        let icon = UiImage::from_rgba(48, 48, vec![255; 48 * 48 * 4]).unwrap();
        let apps = applications(Some(icon));
        let scene = overview_scene(1, Some(&snapshot()), "", Some(&apps));
        assert!(scene.nodes().iter().any(|node| matches!(node,
            UiNode::Image { image, .. } if image.width() == 48 && image.height() == 48
        )));
        let search = overview_scene(2, Some(&snapshot()), "editor", Some(&apps));
        assert!(search.nodes().iter().any(|node| matches!(node,
            UiNode::Image { image, .. } if image.width() == 48 && image.height() == 48
        )));
    }

    #[test]
    fn overview_scene_does_not_require_a_gpu() {
        let scene = UiScene::overview(1, 1920.0, 1080.0);
        assert!(scene.nodes().iter().any(|node| matches!(node,
            UiNode::Panel { color, bounds, .. }
                if *color == Color::BACKGROUND && *bounds == Rect::new(0.0, 0.0, 1920.0, 1080.0)
        )));
        assert!(scene.nodes().iter().any(|node| matches!(node,
            UiNode::Label { text, .. } if text == "Search apps"
        )));
    }
}
