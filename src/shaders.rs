use raylib::prelude::*;
use crate::vertex::Vertex;
use crate::fragment::Fragment;
use crate::Uniforms;

// This function manually multiplies a 4x4 matrix with a 4D vector (in homogeneous coordinates)
fn multiply_matrix_vector4(matrix: &Matrix, vector: &Vector4) -> Vector4 {
    Vector4::new(
        matrix.m0 * vector.x + matrix.m4 * vector.y + matrix.m8 * vector.z + matrix.m12 * vector.w,
        matrix.m1 * vector.x + matrix.m5 * vector.y + matrix.m9 * vector.z + matrix.m13 * vector.w,
        matrix.m2 * vector.x + matrix.m6 * vector.y + matrix.m10 * vector.z + matrix.m14 * vector.w,
        matrix.m3 * vector.x + matrix.m7 * vector.y + matrix.m11 * vector.z + matrix.m15 * vector.w,
    )
}

pub fn vertex_shader(vertex: &Vertex, uniforms: &Uniforms) -> Vertex {
  // Convert vertex position to homogeneous coordinates (Vec4) by adding a w-component of 1.0
  let position_vec4 = Vector4::new(
    vertex.position.x,
    vertex.position.y,
    vertex.position.z,
    1.0
  );

  // Apply Model transformation
  let world_position = multiply_matrix_vector4(&uniforms.model_matrix, &position_vec4);

  // Apply View transformation (camera)
  let view_position = multiply_matrix_vector4(&uniforms.view_matrix, &world_position);

  // Apply Projection transformation (perspective)
  let clip_position = multiply_matrix_vector4(&uniforms.projection_matrix, &view_position);

  // Perform perspective division to get NDC (Normalized Device Coordinates)
  let ndc = if clip_position.w != 0.0 {
      Vector3::new(
          clip_position.x / clip_position.w,
          clip_position.y / clip_position.w,
          clip_position.z / clip_position.w,
      )
  } else {
      Vector3::new(clip_position.x, clip_position.y, clip_position.z)
  };

  // Apply Viewport transformation to get screen coordinates
  let ndc_vec4 = Vector4::new(ndc.x, ndc.y, ndc.z, 1.0);
  let screen_position = multiply_matrix_vector4(&uniforms.viewport_matrix, &ndc_vec4);

  let transformed_position = Vector3::new(
      screen_position.x,
      screen_position.y,
      screen_position.z,
  );

  // Create a new Vertex with the transformed position
  Vertex {
    position: vertex.position,
    normal: vertex.normal,
    tex_coords: vertex.tex_coords,
    color: vertex.color,
    transformed_position,
    transformed_normal: vertex.normal, // Note: Correct normal transformation is more complex
  }
}

// === Animated Fragment Shader Examples ===

/// Example 1: Random flickering colors per fragment
#[allow(dead_code)]
fn shader_random_flicker(fragment: &Fragment, time: f32) -> Vector3 {
    let world_pos = fragment.world_position;
    let base_color = fragment.color;

    // Create pseudo-random values based on position and time
    let seed = world_pos.x * 12.9898 + world_pos.y * 78.233 + world_pos.z * 45.164 + time * 3.0;
    let random = (seed.sin() * 43758.5453).fract();

    let flicker_color = Vector3::new(
        (random * 7.0).sin() * 0.5 + 0.5,
        (random * 11.0).cos() * 0.5 + 0.5,
        (random * 13.0).sin() * 0.5 + 0.5,
    );

    // Mix with base lighting
    Vector3::new(
        base_color.x * 0.5 + flicker_color.x * 0.5,
        base_color.y * 0.5 + flicker_color.y * 0.5,
        base_color.z * 0.5 + flicker_color.z * 0.5,
    )
}

/// Example 2: Horizontal stripes moving upward
#[allow(dead_code)]
fn shader_moving_stripes(fragment: &Fragment, time: f32) -> Vector3 {
    let world_pos = fragment.world_position;
    let base_color = fragment.color;

    // Add time to Y position to make stripes move upward
    let stripe_frequency = 1.0;
    let animated_y = world_pos.y + time * 0.5; // Speed of movement
    let stripe = ((animated_y * stripe_frequency).floor() % 2.0).abs();

    let stripe_color1 = Vector3::new(1.0, 0.3, 0.1); // Orange
    let stripe_color2 = Vector3::new(0.1, 0.3, 1.0); // Blue

    let stripe_color = Vector3::new(
        stripe_color1.x * stripe + stripe_color2.x * (1.0 - stripe),
        stripe_color1.y * stripe + stripe_color2.y * (1.0 - stripe),
        stripe_color1.z * stripe + stripe_color2.z * (1.0 - stripe),
    );

    Vector3::new(
        base_color.x * stripe_color.x,
        base_color.y * stripe_color.y,
        base_color.z * stripe_color.z,
    )
}

/// Example 3: Pulsing color waves
#[allow(dead_code)]
fn shader_pulsing_waves(fragment: &Fragment, time: f32) -> Vector3 {
    let world_pos = fragment.world_position;
    let base_color = fragment.color;

    // Animated sine waves that pulse over time
    let wave1 = (world_pos.x * 3.0 + time * 2.0).sin() * 0.5 + 0.5;
    let wave2 = (world_pos.y * 3.0 + time * 1.5).cos() * 0.5 + 0.5;
    let wave3 = (world_pos.z * 3.0 + time * 2.5).sin() * 0.5 + 0.5;

    let wave_color = Vector3::new(wave1, wave2, wave3);

    Vector3::new(
        base_color.x * 0.6 + wave_color.x * 0.4,
        base_color.y * 0.6 + wave_color.y * 0.4,
        base_color.z * 0.6 + wave_color.z * 0.4,
    )
}

/// Example 4: Rotating rainbow gradient
#[allow(dead_code)]
fn shader_rotating_rainbow(fragment: &Fragment, time: f32) -> Vector3 {
    let world_pos = fragment.world_position;
    let base_color = fragment.color;

    // Create rotating rainbow effect
    let angle = world_pos.x.atan2(world_pos.z) + time;
    let hue = (angle / (2.0 * 3.14159)) % 1.0;

    // Convert hue to RGB (simplified HSV to RGB)
    let rainbow_color = Vector3::new(
        ((hue * 6.0).sin()).abs(),
        ((hue * 6.0 + 2.0).sin()).abs(),
        ((hue * 6.0 + 4.0).sin()).abs(),
    );

    Vector3::new(
        base_color.x * 0.5 + rainbow_color.x * 0.5,
        base_color.y * 0.5 + rainbow_color.y * 0.5,
        base_color.z * 0.5 + rainbow_color.z * 0.5,
    )
}

/// Example 5: Expanding rings from origin
#[allow(dead_code)]
fn shader_expanding_rings(fragment: &Fragment, time: f32) -> Vector3 {
    let world_pos = fragment.world_position;
    let base_color = fragment.color;

    // Distance from origin
    let distance = (world_pos.x * world_pos.x + world_pos.y * world_pos.y + world_pos.z * world_pos.z).sqrt();

    // Animated rings expanding outward
    let ring = (distance * 2.0 - time * 2.0).sin() * 0.5 + 0.5;

    let ring_color = Vector3::new(ring, 1.0 - ring, ring * 0.5);

    Vector3::new(
        base_color.x * 0.5 + ring_color.x * 0.5,
        base_color.y * 0.5 + ring_color.y * 0.5,
        base_color.z * 0.5 + ring_color.z * 0.5,
    )
}

/// Example 6: Breathing/pulsing color intensity
#[allow(dead_code)]
fn shader_breathing(fragment: &Fragment, time: f32) -> Vector3 {
    let base_color = fragment.color;

    // Pulse intensity over time
    let pulse = (time * 2.0).sin() * 0.3 + 0.7; // Range: 0.4 to 1.0

    Vector3::new(
        base_color.x * pulse,
        base_color.y * pulse,
        base_color.z * pulse,
    )
}

/// Example 7: Just pass through the base color (standard lighting only)
#[allow(dead_code)]
fn shader_base_color(fragment: &Fragment, _time: f32) -> Vector3 {
    fragment.color
}

// === Main Fragment Shader ===
pub fn fragment_shader(fragment: &Fragment, uniforms: &Uniforms) -> Vector3 {
    let time = uniforms.time;

    // Using rotating rainbow pattern for a colorful animated effect
    shader_rotating_rainbow(fragment, time)
}