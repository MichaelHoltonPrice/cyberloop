//! Global tooltip plugin.
//!
//! Attach `TooltipContent(String)` + `Button` to any entity. On hover, a
//! tooltip panel appears near the cursor with the text.

use bevy::prelude::*;

use crate::theme;

pub struct TooltipPlugin;

/// System set for the tooltip display system.
/// Schedule tooltip-content–writing systems `.before(TooltipSet)` to avoid
/// access conflicts on `TooltipContent`.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct TooltipSet;

impl Plugin for TooltipPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_tooltip_node)
            .add_systems(Update, update_tooltip.in_set(TooltipSet));
    }
}

/// Attach to any `Button` entity to show a tooltip on hover.
#[derive(Component)]
pub struct TooltipContent(pub String);

#[derive(Component)]
struct TooltipNode;

#[derive(Component)]
struct TooltipTextMarker;

fn spawn_tooltip_node(mut commands: Commands) {
    commands
        .spawn((
            TooltipNode,
            Node {
                position_type: PositionType::Absolute,
                max_width: Val::Px(280.0),
                padding: UiRect::axes(Val::Px(10.0), Val::Px(6.0)),
                border: UiRect::all(Val::Px(1.0)),
                left: Val::Px(-1000.0),
                top: Val::Px(-1000.0),
                ..default()
            },
            BackgroundColor(theme::TOOLTIP_BG),
            BorderColor::all(theme::TOOLTIP_BORDER),
            GlobalZIndex(100),
            Visibility::Hidden,
        ))
        .with_child((
            TooltipTextMarker,
            Text::new(""),
            TextFont { font_size: 13.0, ..default() },
            TextColor(theme::PARCHMENT),
        ));
}

fn update_tooltip(
    hoverable_query: Query<(&Interaction, &TooltipContent), With<Button>>,
    mut tooltip_query: Query<(&mut Node, &mut Visibility, &ComputedNode), With<TooltipNode>>,
    mut text_query: Query<&mut Text, With<TooltipTextMarker>>,
    windows: Query<&Window>,
    ui_scale: Res<UiScale>,
    camera_query: Query<&Camera, With<Camera2d>>,
    targeting: Option<Res<crate::plugins::TargetingState>>,
) {
    let Ok((mut node, mut vis, computed)) = tooltip_query.single_mut() else {
        return;
    };

    // Hide tooltip when targeting arrow is active.
    if let Some(ref ts) = targeting {
        if ts.selected_card_index.is_some() {
            *vis = Visibility::Hidden;
            return;
        }
    }

    let hovered = hoverable_query
        .iter()
        .find(|(interaction, _)| **interaction == Interaction::Hovered);

    let Some((_, content)) = hovered else {
        *vis = Visibility::Hidden;
        return;
    };

    if let Ok(mut text) = text_query.single_mut() {
        if **text != content.0 {
            **text = content.0.clone();
        }
    }

    let Ok(window) = windows.single() else {
        *vis = Visibility::Hidden;
        return;
    };
    let Some(cursor_pos) = window.cursor_position() else {
        *vis = Visibility::Hidden;
        return;
    };

    let scale = ui_scale.0 as f32;

    let viewport_offset = camera_query
        .single()
        .ok()
        .and_then(|camera| camera.viewport.as_ref())
        .map(|vp| {
            let sf = window.scale_factor();
            Vec2::new(
                vp.physical_position.x as f32 / sf,
                vp.physical_position.y as f32 / sf,
            )
        })
        .unwrap_or(Vec2::ZERO);

    let ui_pos = (cursor_pos - viewport_offset) / scale;
    let tooltip_x = ui_pos.x + 12.0;
    let tooltip_y = ui_pos.y + 16.0;

    let isf = computed.inverse_scale_factor();
    let tooltip_w = computed.size.x * isf;
    let tooltip_h = computed.size.y * isf;

    // Use actual window size in UI-coordinate space for boundary checks
    let win_w = window.width() / scale;
    let win_h = window.height() / scale;

    let final_x = if tooltip_x + tooltip_w > win_w {
        (ui_pos.x - 12.0 - tooltip_w).max(0.0)
    } else {
        tooltip_x
    };
    let final_y = if tooltip_y + tooltip_h > win_h {
        (ui_pos.y - 16.0 - tooltip_h).max(0.0)
    } else {
        tooltip_y
    };

    node.left = Val::Px(final_x);
    node.top = Val::Px(final_y);
    *vis = Visibility::Visible;
}
