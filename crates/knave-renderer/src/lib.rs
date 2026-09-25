//! wgpu-backed rendering boundary for Knave UI scenes.

use knave_ui::{Color, Rect, UiNode, UiScene};

#[derive(Clone, Debug, PartialEq)]
pub enum RenderCommand {
    FillRect {
        bounds: Rect,
        color: Color,
    },
    Text {
        bounds: Rect,
        color: Color,
        text: String,
    },
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RenderList {
    pub revision: u64,
    pub commands: Vec<RenderCommand>,
}

impl RenderList {
    pub fn from_scene(scene: &UiScene) -> Self {
        let commands = scene
            .nodes()
            .iter()
            .map(|node| match node {
                UiNode::Panel { bounds, color, .. } => RenderCommand::FillRect {
                    bounds: *bounds,
                    color: *color,
                },
                UiNode::Label {
                    bounds,
                    color,
                    text,
                    ..
                } => RenderCommand::Text {
                    bounds: *bounds,
                    color: *color,
                    text: text.clone(),
                },
            })
            .collect();

        Self {
            revision: scene.revision(),
            commands,
        }
    }
}

pub struct WgpuRenderer {
    instance: wgpu::Instance,
}

impl WgpuRenderer {
    pub fn new() -> Self {
        Self {
            instance: wgpu::Instance::default(),
        }
    }

    pub fn instance(&self) -> &wgpu::Instance {
        &self.instance
    }

    pub fn prepare(&self, scene: &UiScene) -> RenderList {
        RenderList::from_scene(scene)
    }
}

impl Default for WgpuRenderer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scene_conversion_preserves_revision_and_order() {
        let scene = UiScene::bar(4, 1280.0, 36.0);
        let renderer = WgpuRenderer::new();
        let list = renderer.prepare(&scene);

        assert_eq!(list.revision, 4);
        assert_eq!(list.commands.len(), 2);
        assert!(matches!(list.commands[0], RenderCommand::FillRect { .. }));
        assert!(matches!(list.commands[1], RenderCommand::Text { .. }));
    }
}
