use raylib::prelude::*;
use crate::vertex::Vertex;
use crate::fragment::Fragment;
use crate::Uniforms;
use std::f32::consts::PI;

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

/// Vertex Shader Especial para el Sol con Distorsión y Flare
/// Aplica desplazamiento procedural en el vertex shader para simular:
/// - Prominencias solares
/// - Distorsiones de plasma
/// - Efectos de flare visual
pub fn vertex_shader_sun(vertex: &Vertex, uniforms: &Uniforms) -> Vertex {
  let time = uniforms.time;
  
  // Posición original del vértice
  let original_pos = vertex.position;
  
  // ======================================
  // DISTORSIÓN PROCEDURAL EN EL VERTEX SHADER
  // ======================================
  
  // Calcular distorsión basada en la posición y el tiempo
  let distortion_freq = 4.0;
  let distortion_amplitude = 0.08; // 8% del radio
  
  // Múltiples ondas de distorsión para simular actividad solar
  let wave1 = (original_pos.x * distortion_freq + time * 1.2).sin();
  let wave2 = (original_pos.y * distortion_freq + time * 1.5).cos();
  let wave3 = (original_pos.z * distortion_freq + time * 1.0).sin();
  
  // Combinar ondas
  let distortion = (wave1 + wave2 + wave3) / 3.0 * distortion_amplitude;
  
  // Normalizar la posición para obtener la dirección
  let length = (original_pos.x * original_pos.x + 
               original_pos.y * original_pos.y + 
               original_pos.z * original_pos.z).sqrt().max(0.0001);
  let normal_dir = Vector3::new(
      original_pos.x / length,
      original_pos.y / length,
      original_pos.z / length
  );
  
  // Aplicar distorsión a lo largo de la normal (prominencias solares)
  let displaced_position = Vector3::new(
      original_pos.x + normal_dir.x * distortion,
      original_pos.y + normal_dir.y * distortion,
      original_pos.z + normal_dir.z * distortion,
  );
  
  // ======================================
  // FLARE VISUAL: Prominencias Extremas
  // Crear prominencias solares ocasionales más dramáticas
  // ======================================
  
  // Convertir a coordenadas esféricas simplificadas
  let theta = original_pos.y.atan2((original_pos.x * original_pos.x + original_pos.z * original_pos.z).sqrt());
  let phi = original_pos.z.atan2(original_pos.x);
  
  // Patrones de prominencias
  let prominence_pattern = ((theta * 8.0 + time * 0.8).sin() * (phi * 6.0 + time * 0.6).cos()).abs();
  let prominence_intensity = if prominence_pattern > 0.85 {
      (prominence_pattern - 0.85) * 3.0 // Prominencias extremas
  } else {
      0.0
  };
  
  // Posición final con prominencias
  let final_position = Vector3::new(
      displaced_position.x + normal_dir.x * prominence_intensity * 0.15,
      displaced_position.y + normal_dir.y * prominence_intensity * 0.15,
      displaced_position.z + normal_dir.z * prominence_intensity * 0.15,
  );
  
  // ======================================
  // TRANSFORMACIONES ESTÁNDAR
  // ======================================
  
  // Convertir a homogeneous coordinates
  let position_vec4 = Vector4::new(
    final_position.x,
    final_position.y,
    final_position.z,
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
    position: vertex.position, // Mantener posición original para cálculos en fragment shader
    normal: vertex.normal,
    tex_coords: vertex.tex_coords,
    color: vertex.color,
    transformed_position,
    transformed_normal: vertex.normal,
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

// === Planet Shaders ===

/// Helper function to create procedural noise using multiple octaves
fn noise_3d(pos: Vector3, time: f32) -> f32 {
    let scale = 4.0;
    let n1 = (pos.x * scale + time * 0.1).sin() * 0.5 + 0.5;
    let n2 = (pos.y * scale * 1.3 + time * 0.15).sin() * 0.5 + 0.5;
    let n3 = (pos.z * scale * 0.7 + time * 0.12).sin() * 0.5 + 0.5;
    (n1 + n2 + n3) / 3.0
}

/// Helper function to create fractal noise (multiple octaves)
fn fractal_noise(pos: Vector3, time: f32, octaves: i32) -> f32 {
    let mut value = 0.0;
    let mut amplitude = 0.5;
    let mut frequency = 1.0;
    
    for _ in 0..octaves {
        let noise_pos = Vector3::new(
            pos.x * frequency,
            pos.y * frequency,
            pos.z * frequency,
        );
        value += noise_3d(noise_pos, time) * amplitude;
        amplitude *= 0.5;
        frequency *= 2.0;
    }
    
    value
}

/// Helper function to convert spherical coordinates
fn spherical_coords(pos: Vector3) -> (f32, f32, f32) {
    let r = (pos.x * pos.x + pos.y * pos.y + pos.z * pos.z).sqrt();
    let r_xy = (pos.x * pos.x + pos.z * pos.z).sqrt();
    // Evitar división por cero
    let theta = if r_xy > 0.0001 || pos.y.abs() > 0.0001 {
        pos.y.atan2(r_xy) // Latitude
    } else {
        0.0
    };
    let phi = if pos.x.abs() > 0.0001 || pos.z.abs() > 0.0001 {
        pos.z.atan2(pos.x) // Longitude
    } else {
        0.0
    };
    (r.max(0.0001), theta, phi) // Asegurar r > 0
}

/// Planeta 1: Planeta Rocoso (MÚLTIPLES CAPAS)
/// CAPA 1: Ruido fractal para terreno base
/// CAPA 2: Gradientes de altitud simulados
/// CAPA 3: Iluminación simulada con terminador (día/noche)
/// CAPA 4: Efectos de erosión y valles
pub fn shader_rocky_planet(fragment: &Fragment, time: f32) -> Vector3 {
    let world_pos = fragment.world_position;
    let base_color = fragment.color;
    
    // Convertir a coordenadas esféricas para crear patrones
    let (r, theta, _phi) = spherical_coords(world_pos);
    
    // === CAPA 1: Ruido fractal para terreno base ===
    let noise1 = fractal_noise(world_pos, time * 0.1, 4);
    let noise2 = fractal_noise(Vector3::new(world_pos.x * 0.5, world_pos.y * 2.0, world_pos.z * 0.5), time * 0.05, 3);
    let terrain_noise = noise1 * 0.7 + noise2 * 0.3;
    
    // === CAPA 2: Gradientes de altitud simulados ===
    // Simular diferentes altitudes basadas en latitud
    let altitude_gradient = (theta * 2.0).sin() * 0.5 + 0.5; // Más alto en el ecuador
    let altitude_variation = terrain_noise * 0.3 + altitude_gradient * 0.7;
    
    // === CAPA 3: Iluminación simulada con terminador (día/noche) ===
    // Simular posición del sol (luz direccional)
    let sun_dir_raw = Vector3::new(1.0, 0.5, 0.3);
    let sun_dir_len = (sun_dir_raw.x * sun_dir_raw.x + sun_dir_raw.y * sun_dir_raw.y + sun_dir_raw.z * sun_dir_raw.z).sqrt().max(0.0001);
    let sun_direction = Vector3::new(sun_dir_raw.x / sun_dir_len, sun_dir_raw.y / sun_dir_len, sun_dir_raw.z / sun_dir_len);
    let normal = Vector3::new(world_pos.x / r, world_pos.y / r, world_pos.z / r);
    let sun_dot = (normal.x * sun_direction.x + normal.y * sun_direction.y + normal.z * sun_direction.z).max(0.0);
    
    // Terminador (zona crepuscular) más suave
    let terminator = (sun_dot * 3.0 - 1.5).clamp(0.0, 1.0);
    let day_night = sun_dot * 0.7 + 0.3; // Nunca completamente oscuro
    
    // === CAPA 4: Efectos de erosión y valles ===
    let erosion = fractal_noise(Vector3::new(world_pos.x * 3.0, world_pos.y * 3.0, world_pos.z * 3.0), time * 0.02, 2);
    
    // Colores base para planeta rocoso con variaciones
    let rock_color1 = Vector3::new(0.4, 0.3, 0.2); // Marrón oscuro (valles)
    let rock_color2 = Vector3::new(0.5, 0.4, 0.3); // Marrón medio
    let rock_color3 = Vector3::new(0.6, 0.5, 0.4); // Marrón claro (montañas)
    let rock_color4 = Vector3::new(0.35, 0.35, 0.3); // Gris tierra
    let rock_color5 = Vector3::new(0.7, 0.6, 0.5); // Marrón claro (picos)
    
    // Mezclar colores basado en altitud y ruido
    let color_mix = altitude_variation * 0.6 + terrain_noise * 0.4;
    let planet_color = if color_mix < 0.25 {
        // Valles profundos
        Vector3::new(
            rock_color1.x + (rock_color2.x - rock_color1.x) * (color_mix / 0.25),
            rock_color1.y + (rock_color2.y - rock_color1.y) * (color_mix / 0.25),
            rock_color1.z + (rock_color2.z - rock_color1.z) * (color_mix / 0.25),
        )
    } else if color_mix < 0.5 {
        // Terreno medio
        Vector3::new(
            rock_color2.x + (rock_color3.x - rock_color2.x) * ((color_mix - 0.25) / 0.25),
            rock_color2.y + (rock_color3.y - rock_color2.y) * ((color_mix - 0.25) / 0.25),
            rock_color2.z + (rock_color3.z - rock_color2.z) * ((color_mix - 0.25) / 0.25),
        )
    } else if color_mix < 0.75 {
        // Montañas
        Vector3::new(
            rock_color3.x + (rock_color4.x - rock_color3.x) * ((color_mix - 0.5) / 0.25),
            rock_color3.y + (rock_color4.y - rock_color3.y) * ((color_mix - 0.5) / 0.25),
            rock_color3.z + (rock_color4.z - rock_color3.z) * ((color_mix - 0.5) / 0.25),
        )
    } else {
        // Picos
        Vector3::new(
            rock_color4.x + (rock_color5.x - rock_color4.x) * ((color_mix - 0.75) / 0.25),
            rock_color4.y + (rock_color5.y - rock_color4.y) * ((color_mix - 0.75) / 0.25),
            rock_color4.z + (rock_color5.z - rock_color4.z) * ((color_mix - 0.75) / 0.25),
        )
    };
    
    // Aplicar efectos de erosión
    let eroded_color = Vector3::new(
        planet_color.x * (1.0 - erosion * 0.2),
        planet_color.y * (1.0 - erosion * 0.2),
        planet_color.z * (1.0 - erosion * 0.2),
    );
    
    // Aplicar iluminación simulada (día/noche) y terminador
    let final_color = Vector3::new(
        eroded_color.x * day_night * terminator,
        eroded_color.y * day_night * terminator,
        eroded_color.z * day_night * terminator,
    );
    
    // Combinar con iluminación base del sistema
    Vector3::new(
        (final_color.x * 0.8 + base_color.x * 0.2).min(1.0),
        (final_color.y * 0.8 + base_color.y * 0.2).min(1.0),
        (final_color.z * 0.8 + base_color.z * 0.2).min(1.0),
    )
}

/// Planeta 2: Gigante Gaseoso (MÚLTIPLES CAPAS)
/// CAPA 1: Bandas de latitud con gradientes
/// CAPA 2: Ondas de gas turbulentas animadas
/// CAPA 3: Iluminación simulada con gradiente de profundidad
/// CAPA 4: Remolinos y vórtices procedurales
pub fn shader_gas_giant(fragment: &Fragment, time: f32) -> Vector3 {
    let world_pos = fragment.world_position;
    let base_color = fragment.color;
    
    // Convertir a coordenadas esféricas
    let (_r, theta, phi) = spherical_coords(world_pos);
    
    // === CAPA 1: Bandas de latitud con gradientes ===
    let band_frequency = 8.0;
    let band_value = (theta * band_frequency + time * 0.3).sin() * 0.5 + 0.5;
    // Gradiente suave entre bandas
    let band_gradient = (theta * band_frequency * 2.0 + time * 0.3).sin() * 0.3 + 0.7;
    
    // === CAPA 2: Ondas de gas turbulentas animadas ===
    let wave1 = (theta * 12.0 + phi * 6.0 + time * 0.5).sin() * 0.3 + 0.7;
    let wave2 = (theta * 8.0 - phi * 4.0 + time * 0.7).cos() * 0.2 + 0.8;
    let wave3 = (theta * 15.0 + phi * 10.0 + time * 0.6).sin() * 0.15 + 0.85;
    let turbulence = wave1 * wave2 * wave3;
    
    // === CAPA 3: Iluminación simulada con gradiente de profundidad ===
    // Simular profundidad de la atmósfera (más brillante en el centro)
    let depth_factor = (1.0 - (theta.abs() / (PI * 2.0))) * 0.5 + 0.5;
    // Simular iluminación solar
    let sun_dot = (theta.sin() * 0.8 + 0.2).max(0.0);
    let atmospheric_light = depth_factor * sun_dot * 0.8 + 0.2;
    
    // === CAPA 4: Remolinos y vórtices procedurales ===
    let swirl = (phi * 3.0 + theta * 2.0 + time * 0.4).sin() * 0.1 + 1.0;
    let vortex = (phi * 5.0 + theta * 3.0 + time * 0.8).cos() * 0.15 + 0.85;
    let vortex_effect = swirl * vortex;
    
    // Colores típicos de gigante gaseoso con más variación
    let gas_color1 = Vector3::new(0.8, 0.5, 0.2); // Naranja brillante
    let gas_color2 = Vector3::new(0.9, 0.7, 0.3); // Amarillo-naranja
    let gas_color3 = Vector3::new(0.7, 0.4, 0.15); // Naranja oscuro
    let gas_color4 = Vector3::new(0.6, 0.3, 0.1); // Marrón-naranja
    let gas_color5 = Vector3::new(0.95, 0.8, 0.4); // Amarillo claro
    
    // Mezclar colores basado en las múltiples capas
    let color_factor = band_value * band_gradient * turbulence;
    
    let planet_color = if color_factor < 0.2 {
        Vector3::new(
            gas_color4.x + (gas_color3.x - gas_color4.x) * (color_factor / 0.2),
            gas_color4.y + (gas_color3.y - gas_color4.y) * (color_factor / 0.2),
            gas_color4.z + (gas_color3.z - gas_color4.z) * (color_factor / 0.2),
        )
    } else if color_factor < 0.4 {
        Vector3::new(
            gas_color3.x + (gas_color1.x - gas_color3.x) * ((color_factor - 0.2) / 0.2),
            gas_color3.y + (gas_color1.y - gas_color3.y) * ((color_factor - 0.2) / 0.2),
            gas_color3.z + (gas_color1.z - gas_color3.z) * ((color_factor - 0.2) / 0.2),
        )
    } else if color_factor < 0.7 {
        Vector3::new(
            gas_color1.x + (gas_color2.x - gas_color1.x) * ((color_factor - 0.4) / 0.3),
            gas_color1.y + (gas_color2.y - gas_color1.y) * ((color_factor - 0.4) / 0.3),
            gas_color1.z + (gas_color2.z - gas_color1.z) * ((color_factor - 0.4) / 0.3),
        )
    } else {
        Vector3::new(
            gas_color2.x + (gas_color5.x - gas_color2.x) * ((color_factor - 0.7) / 0.3),
            gas_color2.y + (gas_color5.y - gas_color2.y) * ((color_factor - 0.7) / 0.3),
            gas_color2.z + (gas_color5.z - gas_color2.z) * ((color_factor - 0.7) / 0.3),
        )
    };
    
    // Aplicar todas las capas
    let final_color = Vector3::new(
        planet_color.x * atmospheric_light * vortex_effect,
        planet_color.y * atmospheric_light * vortex_effect,
        planet_color.z * atmospheric_light * vortex_effect,
    );
    
    // Combinar con iluminación base
    Vector3::new(
        (final_color.x * 0.7 + base_color.x * 0.3).min(1.0),
        (final_color.y * 0.7 + base_color.y * 0.3).min(1.0),
        (final_color.z * 0.7 + base_color.z * 0.3).min(1.0),
    )
}

/// Planeta 3: Planeta Sci-Fi (MÚLTIPLES CAPAS)
/// CAPA 1: Patrones de energía pulsante con múltiples frecuencias
/// CAPA 2: Redes de circuitos y nodos energéticos
/// CAPA 3: Gradientes de color dinámicos con iluminación simulada
/// CAPA 4: Efectos de brillo y resplandor procedural
pub fn shader_scifi_planet(fragment: &Fragment, time: f32) -> Vector3 {
    let world_pos = fragment.world_position;
    let base_color = fragment.color;
    
    // Convertir a coordenadas esféricas
    let (r, theta, phi) = spherical_coords(world_pos);
    
    // === CAPA 1: Patrones de energía pulsante con múltiples frecuencias ===
    let energy_pulse1 = (time * 2.0 + theta * 10.0).sin() * 0.5 + 0.5;
    let energy_pulse2 = (time * 1.5 + phi * 8.0).cos() * 0.5 + 0.5;
    let energy_pulse3 = (time * 2.5 + theta * 12.0 + phi * 6.0).sin() * 0.3 + 0.7;
    let energy_pulse = (energy_pulse1 + energy_pulse2 + energy_pulse3) / 3.0;
    
    // === CAPA 2: Redes de circuitos y nodos energéticos ===
    let circuit_pattern = ((theta * 20.0).sin() * (phi * 15.0).cos()).abs();
    let circuit_intensity = if circuit_pattern > 0.8 { 1.5 } else { 0.6 };
    // Nodos de energía
    let node_pattern = ((theta * 25.0).sin() * (phi * 20.0).sin()).abs();
    let node_intensity = if node_pattern > 0.9 { 2.0 } else { 1.0 };
    let circuit_effect = circuit_intensity * node_intensity * 0.7 + 0.3;
    
    // === CAPA 3: Gradientes de color dinámicos con iluminación simulada ===
    // Simular iluminación direccional para energía
    let energy_dir = Vector3::new(0.7, 0.5, 0.3);
    let energy_dir_len = (energy_dir.x * energy_dir.x + energy_dir.y * energy_dir.y + energy_dir.z * energy_dir.z).sqrt().max(0.0001);
    let energy_direction = Vector3::new(energy_dir.x / energy_dir_len, energy_dir.y / energy_dir_len, energy_dir.z / energy_dir_len);
    let normal = Vector3::new(world_pos.x / r, world_pos.y / r, world_pos.z / r);
    let energy_light = (normal.x * energy_direction.x + normal.y * energy_direction.y + normal.z * energy_direction.z).max(0.0);
    let energy_shadow = energy_light * 0.6 + 0.4;
    
    // === CAPA 4: Efectos de brillo y resplandor procedural ===
    let glow_pattern = (phi * 8.0 + theta * 6.0 + time * 1.0).sin() * 0.5 + 0.5;
    let glow_intensity = (glow_pattern * 2.0 - 1.0).abs() * 0.5 + 0.5;
    let glow_effect = glow_intensity * 1.3 + 0.7;
    
    // Colores futuristas con más variación
    let scifi_color1 = Vector3::new(0.2, 0.8, 1.0); // Cyan brillante
    let scifi_color2 = Vector3::new(0.8, 0.2, 1.0); // Magenta
    let scifi_color3 = Vector3::new(0.4, 0.3, 0.9); // Púrpura
    let scifi_color4 = Vector3::new(0.1, 0.5, 0.9); // Azul brillante
    let scifi_color5 = Vector3::new(0.9, 0.3, 0.8); // Rosa brillante
    let scifi_color6 = Vector3::new(0.3, 0.9, 0.9); // Cyan claro
    
    // Mezclar colores basado en múltiples patrones
    let color_phase = (theta * 6.0 + phi * 4.0 + time * 0.3).sin() * 0.5 + 0.5;
    let color_variation = energy_pulse * 0.4 + color_phase * 0.6;
    
    let planet_color = if color_variation < 0.2 {
        Vector3::new(
            scifi_color4.x + (scifi_color1.x - scifi_color4.x) * (color_variation / 0.2),
            scifi_color4.y + (scifi_color1.y - scifi_color4.y) * (color_variation / 0.2),
            scifi_color4.z + (scifi_color1.z - scifi_color4.z) * (color_variation / 0.2),
        )
    } else if color_variation < 0.4 {
        Vector3::new(
            scifi_color1.x + (scifi_color2.x - scifi_color1.x) * ((color_variation - 0.2) / 0.2),
            scifi_color1.y + (scifi_color2.y - scifi_color1.y) * ((color_variation - 0.2) / 0.2),
            scifi_color1.z + (scifi_color2.z - scifi_color1.z) * ((color_variation - 0.2) / 0.2),
        )
    } else if color_variation < 0.6 {
        Vector3::new(
            scifi_color2.x + (scifi_color3.x - scifi_color2.x) * ((color_variation - 0.4) / 0.2),
            scifi_color2.y + (scifi_color3.y - scifi_color2.y) * ((color_variation - 0.4) / 0.2),
            scifi_color2.z + (scifi_color3.z - scifi_color2.z) * ((color_variation - 0.4) / 0.2),
        )
    } else if color_variation < 0.8 {
        Vector3::new(
            scifi_color3.x + (scifi_color5.x - scifi_color3.x) * ((color_variation - 0.6) / 0.2),
            scifi_color3.y + (scifi_color5.y - scifi_color3.y) * ((color_variation - 0.6) / 0.2),
            scifi_color3.z + (scifi_color5.z - scifi_color3.z) * ((color_variation - 0.6) / 0.2),
        )
    } else {
        Vector3::new(
            scifi_color5.x + (scifi_color6.x - scifi_color5.x) * ((color_variation - 0.8) / 0.2),
            scifi_color5.y + (scifi_color6.y - scifi_color5.y) * ((color_variation - 0.8) / 0.2),
            scifi_color5.z + (scifi_color6.z - scifi_color5.z) * ((color_variation - 0.8) / 0.2),
        )
    };
    
    // Aplicar todas las capas
    let energy_effect = energy_pulse * circuit_effect;
    let final_color = Vector3::new(
        planet_color.x * energy_shadow * glow_effect * energy_effect,
        planet_color.y * energy_shadow * glow_effect * energy_effect,
        planet_color.z * energy_shadow * glow_effect * energy_effect,
    );
    
    Vector3::new(
        (final_color.x * 0.8 + base_color.x * 0.2).min(1.0),
        (final_color.y * 0.8 + base_color.y * 0.2).min(1.0),
        (final_color.z * 0.8 + base_color.z * 0.2).min(1.0),
    )
}

// === Main Fragment Shader (legacy, no longer used) ===
#[allow(dead_code)]
pub fn fragment_shader(fragment: &Fragment, uniforms: &Uniforms) -> Vector3 {
    let time = uniforms.time;

    // Uncomment one of the shader examples below to see different animated effects!
    // Each shader uses the 'time' uniform to create animations

    // shader_random_flicker(fragment, time)
    // shader_moving_stripes(fragment, time)
    // shader_pulsing_waves(fragment, time)
    // shader_rotating_rainbow(fragment, time)
    // shader_expanding_rings(fragment, time)
    // shader_breathing(fragment, time)
    shader_base_color(fragment, time) // Default: just show the lighting
}

/// Shader para anillos procedurales
/// Simula anillos planetarios con partículas y variaciones de densidad
pub fn shader_rings(fragment: &Fragment, time: f32) -> Vector3 {
    let world_pos = fragment.world_position;
    let base_color = fragment.color;
    
    // Anillos están en el plano XZ, así que usamos distancia radial
    let radial_dist = (world_pos.x * world_pos.x + world_pos.z * world_pos.z).sqrt().max(0.0001);
    
    // Crear bandas de anillos
    let ring_bands = (radial_dist * 8.0 + time * 0.2).sin() * 0.5 + 0.5;
    
    // Variación de densidad
    let density = (radial_dist * 12.0).sin() * 0.3 + 0.7;
    let density_variation = fractal_noise(Vector3::new(world_pos.x, 0.0, world_pos.z), time * 0.1, 3);
    
    // Color de anillos (grises con variaciones)
    let ring_color = Vector3::new(0.6, 0.6, 0.65);
    let ring_color_dark = Vector3::new(0.4, 0.4, 0.45);
    
    let final_density = density * density_variation * ring_bands;
    let color_factor = final_density * 0.6 + 0.4;
    
    let planet_color = Vector3::new(
        ring_color_dark.x + (ring_color.x - ring_color_dark.x) * color_factor,
        ring_color_dark.y + (ring_color.y - ring_color_dark.y) * color_factor,
        ring_color_dark.z + (ring_color.z - ring_color_dark.z) * color_factor,
    );
    
    Vector3::new(
        (planet_color.x * base_color.x * final_density).min(1.0),
        (planet_color.y * base_color.y * final_density).min(1.0),
        (planet_color.z * base_color.z * final_density).min(1.0),
    )
}

/// Shader para luna procedural
/// Simula superficie lunar con cráteres y variaciones
pub fn shader_moon(fragment: &Fragment, time: f32) -> Vector3 {
    let world_pos = fragment.world_position;
    let base_color = fragment.color;
    
    // Cráteres con ruido fractal
    let craters = fractal_noise(world_pos, time * 0.01, 5);
    let crater_depth = (1.0 - craters * 0.5).max(0.3);
    
    // Variaciones de superficie lunar
    let surface_variation = fractal_noise(Vector3::new(world_pos.x * 2.0, world_pos.y * 2.0, world_pos.z * 2.0), time * 0.02, 3);
    
    // Colores lunares (grises)
    let moon_color1 = Vector3::new(0.6, 0.6, 0.65); // Gris claro
    let moon_color2 = Vector3::new(0.5, 0.5, 0.55); // Gris medio
    let moon_color3 = Vector3::new(0.4, 0.4, 0.45); // Gris oscuro
    
    let color_factor = surface_variation * crater_depth;
    
    let planet_color = if color_factor < 0.5 {
        Vector3::new(
            moon_color3.x + (moon_color2.x - moon_color3.x) * (color_factor / 0.5),
            moon_color3.y + (moon_color2.y - moon_color3.y) * (color_factor / 0.5),
            moon_color3.z + (moon_color2.z - moon_color3.z) * (color_factor / 0.5),
        )
    } else {
        Vector3::new(
            moon_color2.x + (moon_color1.x - moon_color2.x) * ((color_factor - 0.5) / 0.5),
            moon_color2.y + (moon_color1.y - moon_color2.y) * ((color_factor - 0.5) / 0.5),
            moon_color2.z + (moon_color1.z - moon_color2.z) * ((color_factor - 0.5) / 0.5),
        )
    };
    
    Vector3::new(
        (planet_color.x * base_color.x * crater_depth).min(1.0),
        (planet_color.y * base_color.y * crater_depth).min(1.0),
        (planet_color.z * base_color.z * crater_depth).min(1.0),
    )
}

/// SOL: Shader Avanzado con Múltiples Capas de Ruido
/// Este shader simula un sol con:
/// - CAPA 1: Ruido Perlin simulado con múltiples octavas (turbulencias solares)
/// - CAPA 2: Ruido Cellular simulado (manchas solares y regiones activas)
/// - CAPA 3: Ruido Simplex simulado (flujos de plasma)
/// - CAPA 4: Emisión variable con picos de energía
/// - CAPA 5: Gradiente de temperatura (color dinámico)
/// - CAPA 6: Corona solar con resplandor
/// - CAPA 7: Llamaradas solares procedurales
pub fn shader_sun(fragment: &Fragment, time: f32) -> Vector3 {
    let world_pos = fragment.world_position;
    let (r, theta, phi) = spherical_coords(world_pos);
    
    // ======================================
    // CAPA 1: RUIDO PERLIN SIMULADO (Turbulencias Solares)
    // Usando múltiples octavas para simular Perlin noise
    // ======================================
    let perlin_octave1 = fractal_noise(world_pos, time * 0.3, 6);
    let perlin_octave2 = fractal_noise(
        Vector3::new(world_pos.x * 2.3, world_pos.y * 2.3, world_pos.z * 2.3),
        time * 0.25,
        4
    );
    let perlin_octave3 = fractal_noise(
        Vector3::new(world_pos.x * 4.7, world_pos.y * 4.7, world_pos.z * 4.7),
        time * 0.4,
        3
    );
    // Combinar octavas con diferentes pesos (simulando Perlin noise real)
    let perlin_turbulence = perlin_octave1 * 0.5 + perlin_octave2 * 0.3 + perlin_octave3 * 0.2;
    
    // ======================================
    // CAPA 2: RUIDO CELLULAR SIMULADO (Manchas Solares)
    // Simulando el patrón de Voronoi/Cellular noise
    // ======================================
    let cell_scale = 8.0;
    let cell_x = (world_pos.x * cell_scale + time * 0.1).floor();
    let cell_y = (world_pos.y * cell_scale + time * 0.08).floor();
    let cell_z = (world_pos.z * cell_scale + time * 0.12).floor();
    
    // Generar "puntos de celda" procedurales
    let cell_seed = cell_x * 127.1 + cell_y * 311.7 + cell_z * 74.7;
    let cell_random = (cell_seed.sin() * 43758.5453).fract();
    
    // Distancia al centro de la celda (simula manchas solares)
    let cell_dist = ((world_pos.x * cell_scale - cell_x).abs() + 
                     (world_pos.y * cell_scale - cell_y).abs() + 
                     (world_pos.z * cell_scale - cell_z).abs()) * 0.5;
    let cellular_pattern = (1.0 - cell_dist.min(1.0)) * cell_random;
    
    // Manchas solares (regiones más oscuras y frías)
    let sunspot_threshold = 0.7;
    let is_sunspot = if cellular_pattern > sunspot_threshold { 
        0.4 + cellular_pattern * 0.3 
    } else { 
        1.0 
    };
    
    // ======================================
    // CAPA 3: RUIDO SIMPLEX SIMULADO (Flujos de Plasma)
    // Simulando el patrón de flujo de Simplex noise
    // ======================================
    let simplex_freq1 = 3.5;
    let simplex_freq2 = 7.2;
    let simplex_flow1 = ((world_pos.x * simplex_freq1 + time * 0.5).sin() * 
                         (world_pos.y * simplex_freq1 - time * 0.4).cos() *
                         (world_pos.z * simplex_freq1 + time * 0.6).sin()) * 0.5 + 0.5;
    let simplex_flow2 = ((world_pos.x * simplex_freq2 - time * 0.7).cos() * 
                         (world_pos.y * simplex_freq2 + time * 0.5).sin() *
                         (world_pos.z * simplex_freq2 - time * 0.8).cos()) * 0.5 + 0.5;
    let simplex_plasma = simplex_flow1 * 0.6 + simplex_flow2 * 0.4;
    
    // ======================================
    // CAPA 4: EMISIÓN VARIABLE (Picos de Energía)
    // Simula la variación en la emisión solar
    // ======================================
    // Pulsación base del sol
    let base_pulse = (time * 1.5).sin() * 0.15 + 0.85; // Rango: 0.7 - 1.0
    
    // Picos de energía localizados (usando coordenadas esféricas)
    let energy_spike1 = ((theta * 8.0 + time * 2.0).sin() * (phi * 6.0 + time * 1.5).cos()).abs();
    let energy_spike2 = ((theta * 12.0 - time * 1.8).cos() * (phi * 10.0 - time * 2.2).sin()).abs();
    let energy_spikes = (energy_spike1 + energy_spike2) * 0.5;
    
    // Crear regiones de alta energía (flares)
    let high_energy_regions = if energy_spikes > 0.75 {
        1.0 + (energy_spikes - 0.75) * 4.0 // Picos intensos
    } else {
        1.0
    };
    
    // Combinar emisiones
    let emission_intensity = base_pulse * high_energy_regions * (0.8 + perlin_turbulence * 0.4);
    
    // ======================================
    // CAPA 5: GRADIENTE DE TEMPERATURA (Color Dinámico)
    // Simula diferentes temperaturas en la superficie solar
    // ======================================
    // Temperatura base calculada desde el centro hacia el borde
    let distance_from_center = (world_pos.x * world_pos.x + 
                                world_pos.y * world_pos.y + 
                                world_pos.z * world_pos.z).sqrt();
    let radial_gradient = 1.0 - (distance_from_center / r).min(1.0);
    
    // Temperatura variando con el ruido y el tiempo
    let temperature = radial_gradient * 0.4 + 
                     perlin_turbulence * 0.3 + 
                     simplex_plasma * 0.2 + 
                     cellular_pattern * 0.1;
    
    // Definir colores basados en temperatura (negro de cuerpo)
    // Temperaturas más altas = más blanco/azul
    // Temperaturas medias = amarillo/naranja
    // Temperaturas bajas = rojo/naranja oscuro
    let temp_hot = Vector3::new(1.0, 0.95, 0.8);      // Blanco-amarillo (centro, muy caliente)
    let temp_medium = Vector3::new(1.0, 0.7, 0.2);    // Amarillo-naranja (medio)
    let temp_warm = Vector3::new(1.0, 0.5, 0.1);      // Naranja (caliente)
    let temp_cool = Vector3::new(0.9, 0.3, 0.05);     // Rojo-naranja (relativamente frío)
    let temp_sunspot = Vector3::new(0.4, 0.15, 0.05); // Rojo oscuro (manchas solares)
    
    // Gradiente de temperatura con transiciones suaves
    let base_color = if temperature > 0.8 {
        // Región muy caliente (centro)
        let t = (temperature - 0.8) / 0.2;
        Vector3::new(
            temp_medium.x + (temp_hot.x - temp_medium.x) * t,
            temp_medium.y + (temp_hot.y - temp_medium.y) * t,
            temp_medium.z + (temp_hot.z - temp_medium.z) * t,
        )
    } else if temperature > 0.6 {
        // Región caliente
        let t = (temperature - 0.6) / 0.2;
        Vector3::new(
            temp_warm.x + (temp_medium.x - temp_warm.x) * t,
            temp_warm.y + (temp_medium.y - temp_warm.y) * t,
            temp_warm.z + (temp_medium.z - temp_warm.z) * t,
        )
    } else if temperature > 0.4 {
        // Región media
        let t = (temperature - 0.4) / 0.2;
        Vector3::new(
            temp_cool.x + (temp_warm.x - temp_cool.x) * t,
            temp_cool.y + (temp_warm.y - temp_cool.y) * t,
            temp_cool.z + (temp_warm.z - temp_cool.z) * t,
        )
    } else {
        // Región más fría
        let t = temperature / 0.4;
        Vector3::new(
            temp_sunspot.x + (temp_cool.x - temp_sunspot.x) * t,
            temp_sunspot.y + (temp_cool.y - temp_sunspot.y) * t,
            temp_sunspot.z + (temp_sunspot.z - temp_sunspot.z) * t,
        )
    };
    
    // ======================================
    // CAPA 6: CORONA SOLAR (Resplandor en los Bordes)
    // Simula la corona solar visible en los bordes
    // ======================================
    let normal = Vector3::new(world_pos.x / r, world_pos.y / r, world_pos.z / r);
    // Simular vista desde la cámara (aproximación)
    let view_dir = Vector3::new(0.0, 0.0, 1.0); // Vista simplificada
    let view_dot = (normal.x * view_dir.x + normal.y * view_dir.y + normal.z * view_dir.z).abs();
    
    // Efecto limbo (más brillante en los bordes)
    let limb_darkening = view_dot.powf(0.4); // Exponente < 1 para efecto inverso en bordes
    let corona_brightness = (1.0 - limb_darkening) * 0.5 + 0.5;
    
    // Corona animada
    let corona_variation = (time * 0.8 + theta * 4.0).sin() * 0.2 + 0.8;
    let corona_effect = corona_brightness * corona_variation * 1.3;
    
    // ======================================
    // CAPA 7: LLAMARADAS SOLARES (Flares Procedurales)
    // Simula erupciones solares dramáticas
    // ======================================
    let flare_pattern = ((theta * 15.0 + time * 3.0).sin() * 
                         (phi * 12.0 - time * 2.5).cos()).abs();
    let flare_intensity = if flare_pattern > 0.85 {
        let flare_strength = (flare_pattern - 0.85) / 0.15;
        let flare_pulse = (time * 5.0).sin() * 0.5 + 0.5;
        1.0 + flare_strength * flare_pulse * 2.0 // Puede aumentar hasta 3x
    } else {
        1.0
    };
    
    // ======================================
    // COMBINACIÓN FINAL DE TODAS LAS CAPAS
    // ======================================
    let combined_color = Vector3::new(
        base_color.x * emission_intensity * is_sunspot * corona_effect * flare_intensity,
        base_color.y * emission_intensity * is_sunspot * corona_effect * flare_intensity,
        base_color.z * emission_intensity * is_sunspot * corona_effect * flare_intensity,
    );
    
    // Intensidad mínima para que el sol siempre sea visible y brillante
    let min_intensity = 0.5;
    Vector3::new(
        (combined_color.x.max(min_intensity) * 1.5).min(3.0), // Permitir valores > 1.0 para efecto HDR
        (combined_color.y.max(min_intensity * 0.5) * 1.5).min(3.0),
        (combined_color.z.max(min_intensity * 0.2) * 1.5).min(3.0),
    )
}

/// Fragment shader with planet type selection
pub fn fragment_shader_planet(fragment: &Fragment, uniforms: &Uniforms, planet_type: PlanetType) -> Vector3 {
    let time = uniforms.time;
    
    match planet_type {
        PlanetType::Rocky => shader_rocky_planet(fragment, time),
        PlanetType::GasGiant => shader_gas_giant(fragment, time),
        PlanetType::SciFi => shader_scifi_planet(fragment, time),
        PlanetType::Ice => shader_ice_planet(fragment, time),
        PlanetType::Volcanic => shader_volcanic_planet(fragment, time),
        PlanetType::Ring => shader_rings(fragment, time),
        PlanetType::Moon => shader_moon(fragment, time),
        PlanetType::Sun => shader_sun(fragment, time),
        PlanetType::Ship => shader_ship(fragment, time),
    }
}

/// Planeta 4: Planeta Helado (MÚLTIPLES CAPAS)
/// CAPA 1: Superficie de hielo con fracturas
/// CAPA 2: Capas de nieve con gradientes de profundidad
/// CAPA 3: Iluminación simulada con reflexión de hielo
/// CAPA 4: Efectos de cristales y escarcha
pub fn shader_ice_planet(fragment: &Fragment, time: f32) -> Vector3 {
    let world_pos = fragment.world_position;
    let base_color = fragment.color;
    
    let (r, theta, phi) = spherical_coords(world_pos);
    
    // === CAPA 1: Superficie de hielo con fracturas ===
    let ice_fracture = fractal_noise(world_pos, time * 0.05, 5);
    let crack_pattern = (phi * 12.0 + theta * 8.0 + time * 0.2).sin() * 0.3 + 0.7;
    
    // === CAPA 2: Capas de nieve con gradientes de profundidad ===
    let snow_depth = (theta * 3.0).sin() * 0.5 + 0.5;
    let snow_layers = fractal_noise(Vector3::new(world_pos.x * 2.0, world_pos.y * 2.0, world_pos.z * 2.0), time * 0.03, 3);
    let snow_gradient = snow_depth * 0.6 + snow_layers * 0.4;
    
    // === CAPA 3: Iluminación simulada con reflexión de hielo ===
    let ice_dir = Vector3::new(0.8, 0.4, 0.2);
    let ice_dir_len = (ice_dir.x * ice_dir.x + ice_dir.y * ice_dir.y + ice_dir.z * ice_dir.z).sqrt().max(0.0001);
    let ice_direction = Vector3::new(ice_dir.x / ice_dir_len, ice_dir.y / ice_dir_len, ice_dir.z / ice_dir_len);
    let normal = Vector3::new(world_pos.x / r, world_pos.y / r, world_pos.z / r);
    let ice_reflection = (normal.x * ice_direction.x + normal.y * ice_direction.y + normal.z * ice_direction.z).max(0.0);
    let ice_shine = ice_reflection * 1.5 + 0.5; // Brillo de hielo
    
    // === CAPA 4: Efectos de cristales y escarcha ===
    let crystal_pattern = ((theta * 30.0).sin() * (phi * 25.0).cos()).abs();
    let frost_effect = (phi * 10.0 + theta * 8.0 + time * 0.4).sin() * 0.2 + 0.8;
    let crystal_glow = if crystal_pattern > 0.95 { 1.8 } else { 1.0 };
    
    // Colores de hielo y nieve
    let ice_color1 = Vector3::new(0.9, 0.95, 1.0); // Blanco azulado
    let ice_color2 = Vector3::new(0.7, 0.85, 0.95); // Azul claro
    let ice_color3 = Vector3::new(0.5, 0.7, 0.9); // Azul medio
    let ice_color4 = Vector3::new(0.8, 0.9, 0.98); // Blanco nieve
    let ice_color5 = Vector3::new(0.6, 0.8, 0.95); // Azul hielo
    
    let color_factor = ice_fracture * 0.4 + snow_gradient * 0.6;
    
    let planet_color = if color_factor < 0.25 {
        Vector3::new(
            ice_color3.x + (ice_color2.x - ice_color3.x) * (color_factor / 0.25),
            ice_color3.y + (ice_color2.y - ice_color3.y) * (color_factor / 0.25),
            ice_color3.z + (ice_color2.z - ice_color3.z) * (color_factor / 0.25),
        )
    } else if color_factor < 0.5 {
        Vector3::new(
            ice_color2.x + (ice_color1.x - ice_color2.x) * ((color_factor - 0.25) / 0.25),
            ice_color2.y + (ice_color1.y - ice_color2.y) * ((color_factor - 0.25) / 0.25),
            ice_color2.z + (ice_color1.z - ice_color2.z) * ((color_factor - 0.25) / 0.25),
        )
    } else if color_factor < 0.75 {
        Vector3::new(
            ice_color1.x + (ice_color4.x - ice_color1.x) * ((color_factor - 0.5) / 0.25),
            ice_color1.y + (ice_color4.y - ice_color1.y) * ((color_factor - 0.5) / 0.25),
            ice_color1.z + (ice_color4.z - ice_color1.z) * ((color_factor - 0.5) / 0.25),
        )
    } else {
        Vector3::new(
            ice_color4.x + (ice_color5.x - ice_color4.x) * ((color_factor - 0.75) / 0.25),
            ice_color4.y + (ice_color5.y - ice_color4.y) * ((color_factor - 0.75) / 0.25),
            ice_color4.z + (ice_color5.z - ice_color4.z) * ((color_factor - 0.75) / 0.25),
        )
    };
    
    let final_color = Vector3::new(
        planet_color.x * ice_shine * crystal_glow * frost_effect * crack_pattern,
        planet_color.y * ice_shine * crystal_glow * frost_effect * crack_pattern,
        planet_color.z * ice_shine * crystal_glow * frost_effect * crack_pattern,
    );
    
    Vector3::new(
        (final_color.x * 0.8 + base_color.x * 0.2).min(1.0),
        (final_color.y * 0.8 + base_color.y * 0.2).min(1.0),
        (final_color.z * 0.8 + base_color.z * 0.2).min(1.0),
    )
}

/// Nave Espacial: Shader Gris Mejorado para Visibilidad
/// Shader optimizado pero con mejor visibilidad para la nave
pub fn shader_ship(fragment: &Fragment, _time: f32) -> Vector3 {
    let base_color = fragment.color;
    
    // Color gris metálico más brillante para mejor visibilidad
    let ship_gray = Vector3::new(0.7, 0.7, 0.75); // Gris metálico más claro
    
    // Aplicar iluminación con un mínimo de brillo para asegurar visibilidad
    let min_brightness = 0.3; // Brillo mínimo para que siempre sea visible
    let brightness = base_color.x.max(base_color.y).max(base_color.z);
    let final_brightness = brightness.max(min_brightness);
    
    // Color final con mejor contraste
    Vector3::new(
        (ship_gray.x * final_brightness * 1.2).min(1.0),
        (ship_gray.y * final_brightness * 1.2).min(1.0),
        (ship_gray.z * final_brightness * 1.2).min(1.0),
    )
}

/// Planeta 5: Planeta Volcánico (MÚLTIPLES CAPAS)
/// CAPA 1: Superficie de lava y roca fundida
/// CAPA 2: Flujos de lava animados
/// CAPA 3: Iluminación simulada de lava incandescente
/// CAPA 4: Efectos de humo y ceniza
pub fn shader_volcanic_planet(fragment: &Fragment, time: f32) -> Vector3 {
    let world_pos = fragment.world_position;
    let base_color = fragment.color;
    
    let (_r, theta, phi) = spherical_coords(world_pos);
    
    // === CAPA 1: Superficie de lava y roca fundida ===
    let lava_noise = fractal_noise(world_pos, time * 0.2, 4);
    
    // === CAPA 2: Flujos de lava animados ===
    let lava_flow1 = (theta * 8.0 + phi * 6.0 + time * 0.8).sin() * 0.5 + 0.5;
    let lava_flow2 = (theta * 12.0 - phi * 4.0 + time * 1.0).cos() * 0.3 + 0.7;
    let lava_flow = lava_flow1 * lava_flow2;
    
    // === CAPA 3: Iluminación simulada de lava incandescente ===
    let lava_glow = (time * 3.0 + theta * 5.0).sin() * 0.3 + 0.7;
    let incandescent = lava_glow * 1.5 + 0.5;
    
    // === CAPA 4: Efectos de humo y ceniza ===
    let smoke_pattern = fractal_noise(Vector3::new(world_pos.x * 1.5, world_pos.y * 2.0, world_pos.z * 1.5), time * 0.15, 2);
    let ash_layer = (theta * 4.0 + time * 0.5).sin() * 0.2 + 0.8;
    
    // Colores volcánicos
    let lava_color1 = Vector3::new(1.0, 0.3, 0.0); // Rojo lava
    let lava_color3 = Vector3::new(0.6, 0.2, 0.1); // Rojo oscuro
    let lava_color4 = Vector3::new(0.4, 0.15, 0.1); // Marrón rojizo
    let lava_color5 = Vector3::new(0.8, 0.4, 0.2); // Naranja oscuro
    
    let color_factor = lava_noise * 0.5 + lava_flow * 0.5;
    let is_lava = if color_factor > 0.6 { 1.0 } else { 0.3 };
    
    let planet_color = if color_factor < 0.3 {
        Vector3::new(
            lava_color4.x + (lava_color3.x - lava_color4.x) * (color_factor / 0.3),
            lava_color4.y + (lava_color3.y - lava_color4.y) * (color_factor / 0.3),
            lava_color4.z + (lava_color3.z - lava_color4.z) * (color_factor / 0.3),
        )
    } else if color_factor < 0.6 {
        Vector3::new(
            lava_color3.x + (lava_color5.x - lava_color3.x) * ((color_factor - 0.3) / 0.3),
            lava_color3.y + (lava_color5.y - lava_color3.y) * ((color_factor - 0.3) / 0.3),
            lava_color3.z + (lava_color5.z - lava_color3.z) * ((color_factor - 0.3) / 0.3),
        )
    } else {
        Vector3::new(
            lava_color5.x + (lava_color1.x - lava_color5.x) * ((color_factor - 0.6) / 0.4),
            lava_color5.y + (lava_color1.y - lava_color5.y) * ((color_factor - 0.6) / 0.4),
            lava_color5.z + (lava_color1.z - lava_color5.z) * ((color_factor - 0.6) / 0.4),
        )
    };
    
    let final_color = Vector3::new(
        planet_color.x * incandescent * is_lava * (1.0 - smoke_pattern * 0.3) * ash_layer,
        planet_color.y * incandescent * is_lava * (1.0 - smoke_pattern * 0.3) * ash_layer,
        planet_color.z * incandescent * is_lava * (1.0 - smoke_pattern * 0.3) * ash_layer,
    );
    
    Vector3::new(
        (final_color.x * 0.8 + base_color.x * 0.2).min(1.0),
        (final_color.y * 0.8 + base_color.y * 0.2).min(1.0),
        (final_color.z * 0.8 + base_color.z * 0.2).min(1.0),
    )
}

/// Enum para seleccionar el tipo de planeta
#[derive(Clone, Copy, Debug)]
pub enum PlanetType {
    Rocky,      // Planeta rocoso
    GasGiant,   // Gigante gaseoso
    SciFi,      // Planeta sci-fi
    Ice,        // Planeta helado (adicional)
    Volcanic,   // Planeta volcánico (adicional)
    Ring,       // Para anillos (usa shader especial)
    Moon,       // Para luna (usa shader especial)
    Sun,        // Para el sol (shader especial avanzado)
    Ship,       // Para la nave espacial (shader gris eficiente)
}