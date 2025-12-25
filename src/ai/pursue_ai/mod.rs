pub mod movement;
pub mod wander;

use bevy::{
    app::{App, Plugin, Update},
    ecs::{
        component::Component,
        query::With,
        system::{ParamSet, Query, Res},
    },
    math::Vec3Swizzles,
    transform::components::Transform,
};

use super::pathfinding::PathfindingGraph;
use super::platformer_ai::AIPhysics;

pub const PURSUE_AI_AGENT_RADIUS: f32 = 8.0;

pub enum PursueAIState {
    Wander,
    Pursue,
    Search,
    Attack,
}

pub struct PursueAIPlugin;

impl Plugin for PursueAIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, s_pursue_ai_update);
    }
}

#[derive(Component)]
pub struct PursueAI {
    pub state: PursueAIState,
    pub current_wander_goal: Option<usize>,
}

pub fn s_pursue_ai_update(
    mut queries: ParamSet<(
        Query<(&mut Transform, &mut AIPhysics, &mut PursueAI)>,
        Query<&Transform, With<crate::Player>>,
    )>,
    pathfinding: Res<PathfindingGraph>,
) {
    // Get player position for detection (read-only query)
    let player_pos = queries.p1().single().map(|t| t.translation.xy()).ok();

    // Process AI entities (mutable query)
    for (mut transform, mut physics, mut pursue_ai) in queries.p0().iter_mut() {
        let ai_pos = transform.translation.xy();
        
        // Simple distance-based detection: if player is within range, pursue
        const DETECTION_RANGE_SQ: f32 = 500.0 * 500.0; // 500 pixels detection range
        
        let should_pursue = if let Some(player_position) = player_pos {
            let distance_sq = (ai_pos - player_position).length_squared();
            distance_sq <= DETECTION_RANGE_SQ
        } else {
            false
        };

        let next_state: Option<PursueAIState> = match pursue_ai.state {
            PursueAIState::Wander => {
                if should_pursue {
                    // Transition to Pursue when player detected
                    Some(PursueAIState::Pursue)
                } else {
                    // Continue wandering
                    wander::wander_update(
                        &mut transform,
                        &mut physics,
                        &mut pursue_ai,
                        pathfinding.as_ref(),
                    )
                }
            }
            PursueAIState::Pursue => {
                if !should_pursue {
                    // Transition back to Wander if player is out of range
                    Some(PursueAIState::Wander)
                } else {
                    // Continue pursuing
                    None
                }
            }
            // PursueAIState::Search => {}
            // PursueAIState::Attack => {}
            _ => None,
        };

        if let Some(new_state) = next_state {
            pursue_ai.state = new_state;
        }
    }
}

