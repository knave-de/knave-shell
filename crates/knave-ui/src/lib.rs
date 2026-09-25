//! Renderer-independent primitives for the Knave shell UI.

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
}

impl UiScene {
    pub fn new(revision: u64) -> Self {
        Self {
            revision,
            nodes: Vec::new(),
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

    pub fn overview(revision: u64, width: f32, height: f32) -> Self {
        let mut scene = Self::new(revision);
        scene.push(UiNode::Panel {
            id: NodeId(1),
            bounds: Rect::new(0.0, 0.0, width, height),
            color: Color::BACKGROUND,
        });
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
}
