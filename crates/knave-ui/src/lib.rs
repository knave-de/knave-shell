//! Renderer-independent primitives for the Knave shell UI.

use knave_desktop_api::{DesktopSnapshot, WindowId, WindowSummary, WorkspaceId};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
    pub alpha: u8,
}

impl Color {
    pub const BACKGROUND: Self = Self::rgba(21, 29, 40, 255);
    pub const ACCENT: Self = Self::rgba(93, 173, 226, 255);
    pub const TEXT: Self = Self::rgba(240, 244, 248, 255);

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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NodeId(pub u64);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiAction {
    CloseOverview,
    FocusWorkspace(WorkspaceId),
    FocusWindow(WindowId),
    RestoreWindow(WindowId),
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
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct UiScene {
    revision: u64,
    nodes: Vec<UiNode>,
    targets: Vec<HitTarget>,
}

const MAX_OVERVIEW_WINDOWS: usize = 32;
const OVERVIEW_COLUMNS: usize = 4;

fn window_label(window: &WindowSummary) -> String {
    let label = if window.title.is_empty() {
        window.app_id.as_str()
    } else {
        window.title.as_str()
    };
    label.chars().take(48).collect()
}

impl UiScene {
    pub fn new(revision: u64) -> Self {
        Self {
            revision,
            nodes: Vec::new(),
            targets: Vec::new(),
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

    fn target(&mut self, bounds: Rect, action: UiAction) {
        self.targets.push(HitTarget { bounds, action });
    }
    pub fn bar(revision: u64, width: f32, height: f32) -> Self {
        let mut scene = Self::new(revision);
        scene.push(UiNode::Panel {
            id: NodeId(1),
            bounds: Rect::new(0.0, 0.0, width, height),
            color: Color::BACKGROUND,
        });
        scene.push(UiNode::Label {
            id: NodeId(2),
            bounds: Rect::new(16.0, 0.0, width - 32.0, height),
            color: Color::TEXT,
            text: "Knave".into(),
        });
        scene
    }

    pub fn bar_with_snapshot(
        revision: u64,
        width: f32,
        height: f32,
        snapshot: Option<&DesktopSnapshot>,
    ) -> Self {
        let mut scene = Self::bar(revision, width, height);
        if let Some(UiNode::Label { text, .. }) = scene.nodes.get_mut(1) {
            *text = snapshot
                .and_then(|snapshot| {
                    snapshot
                        .workspaces
                        .iter()
                        .find(|workspace| workspace.active)
                })
                .map_or_else(
                    || "Knave".into(),
                    |workspace| format!("Workspace {}", workspace.workspace.0),
                );
        }
        if let Some(snapshot) = snapshot {
            for (index, workspace) in snapshot.workspaces.iter().enumerate() {
                let bounds = Rect::new(150.0 + index as f32 * 38.0, 5.0, 32.0, height - 10.0);
                scene.push(UiNode::Label {
                    id: NodeId(10 + index as u64),
                    bounds,
                    color: if workspace.active {
                        Color::ACCENT
                    } else {
                        Color::TEXT
                    },
                    text: workspace.workspace.0.to_string(),
                });
                scene.target(bounds, UiAction::FocusWorkspace(workspace.workspace));
            }
        }
        scene
    }

    pub fn overview(revision: u64, width: f32, height: f32) -> Self {
        let mut scene = Self::new(revision);
        scene.push(UiNode::Panel {
            id: NodeId(1),
            bounds: Rect::new(0.0, 0.0, width, height),
            color: Color::BACKGROUND,
        });
        scene.target(Rect::new(0.0, 0.0, width, height), UiAction::CloseOverview);
        scene
    }
    pub fn overview_with_snapshot(
        revision: u64,
        width: f32,
        height: f32,
        snapshot: Option<&DesktopSnapshot>,
    ) -> Self {
        let mut scene = Self::overview(revision, width, height);
        if let Some(snapshot) = snapshot
            && let Some(workspace) = snapshot
                .workspaces
                .iter()
                .find(|workspace| workspace.active)
        {
            scene.push(UiNode::Label {
                id: NodeId(100),
                bounds: Rect::new(32.0, 64.0, width - 64.0, height - 96.0),
                color: Color::TEXT,
                text: format!(
                    "Workspace {} · {} windows",
                    workspace.workspace.0, workspace.window_count
                ),
            });
            for (index, workspace) in snapshot.workspaces.iter().enumerate() {
                let bounds = Rect::new(32.0 + index as f32 * 76.0, 20.0, 68.0, 30.0);
                scene.push(UiNode::Label {
                    id: NodeId(110 + index as u64),
                    bounds,
                    color: if workspace.active {
                        Color::ACCENT
                    } else {
                        Color::TEXT
                    },
                    text: workspace.workspace.0.to_string(),
                });
                scene.target(bounds, UiAction::FocusWorkspace(workspace.workspace));
            }

            let gap = 16.0;
            let card_width = ((width - 64.0 - gap * (OVERVIEW_COLUMNS as f32 - 1.0))
                / OVERVIEW_COLUMNS as f32)
                .max(1.0);
            let card_height = 72.0;
            for (index, window) in snapshot
                .windows
                .iter()
                .filter(|window| window.workspace == workspace.workspace)
                .take(MAX_OVERVIEW_WINDOWS)
                .enumerate()
            {
                let column = index % OVERVIEW_COLUMNS;
                let row = index / OVERVIEW_COLUMNS;
                let bounds = Rect::new(
                    32.0 + column as f32 * (card_width + gap),
                    112.0 + row as f32 * (card_height + gap),
                    card_width,
                    card_height,
                );
                scene.push(UiNode::Panel {
                    id: NodeId(1_000 + index as u64),
                    bounds,
                    color: if window.focused {
                        Color::ACCENT
                    } else {
                        Color::rgba(35, 48, 64, 255)
                    },
                });
                scene.push(UiNode::Label {
                    id: NodeId(2_000 + index as u64),
                    bounds: Rect::new(
                        bounds.x + 12.0,
                        bounds.y + 8.0,
                        (bounds.width - 24.0).max(1.0),
                        (bounds.height - 16.0).max(1.0),
                    ),
                    color: Color::TEXT,
                    text: window_label(window),
                });
                scene.target(
                    bounds,
                    if window.minimized {
                        UiAction::RestoreWindow(window.id)
                    } else {
                        UiAction::FocusWindow(window.id)
                    },
                );
            }
        }
        scene
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bar_scene_is_renderer_independent() {
        let scene = UiScene::bar(7, 1920.0, 36.0);
        assert_eq!(scene.revision(), 7);
        assert_eq!(scene.nodes().len(), 2);
    }

    #[test]
    fn overview_scene_does_not_require_a_gpu() {
        let scene = UiScene::overview(1, 1920.0, 1080.0);
        assert_eq!(scene.nodes().len(), 1);
        assert_eq!(
            scene.nodes()[0],
            UiNode::Panel {
                id: NodeId(1),
                bounds: Rect::new(0.0, 0.0, 1920.0, 1080.0),
                color: Color::BACKGROUND,
            }
        );
    }
    #[test]
    fn snapshot_scene_reflects_active_workspace() {
        let snapshot = DesktopSnapshot {
            generation: 4,
            workspaces: vec![knave_desktop_api::WorkspaceSummary {
                workspace: knave_desktop_api::WorkspaceId(2),
                active: true,
                window_count: 3,
                visible_window_count: 2,
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
        };
        let bar = UiScene::bar_with_snapshot(4, 1920.0, 36.0, Some(&snapshot));
        assert!(matches!(&bar.nodes()[1], UiNode::Label { text, .. } if text == "Workspace 2"));
        let overview = UiScene::overview_with_snapshot(4, 1920.0, 1080.0, Some(&snapshot));
        assert_eq!(overview.nodes().len(), 7);
        assert_eq!(
            overview.hit_test(40.0, 120.0),
            Some(UiAction::FocusWindow(WindowId(41)))
        );
        assert_eq!(
            overview.hit_test(500.0, 120.0),
            Some(UiAction::RestoreWindow(WindowId(42)))
        );
        assert_eq!(
            overview.hit_test(40.0, 25.0),
            Some(UiAction::FocusWorkspace(WorkspaceId(2)))
        );
        assert_eq!(
            overview.hit_test(1000.0, 700.0),
            Some(UiAction::CloseOverview)
        );
    }
}
