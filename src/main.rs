// main.rs

mod framebuffer;
mod triangle;
mod line;
mod vertex;
mod fragment;
mod shaders;
mod obj;
mod matrix;
mod camera;
mod light;

use crate::matrix::{create_model_matrix, create_projection_matrix, create_viewport_matrix};
use crate::camera::Camera;
use crate::light::Light;
use framebuffer::Framebuffer;
use vertex::Vertex;
use triangle::triangle;
use shaders::{vertex_shader, vertex_shader_sun, fragment_shader_planet, PlanetType};
use obj::Obj;
use raylib::prelude::*;
use std::thread;
use std::time::Duration;
use std::f32::consts::PI;

pub struct Uniforms {
    pub model_matrix: Matrix,
    pub view_matrix: Matrix,
    pub projection_matrix: Matrix,
    pub viewport_matrix: Matrix,
    pub time: f32,
}

// Estructura para representar un planeta en el sistema solar
struct Planet {
    orbital_radius: f32,      // Radio de la órbita
    orbital_angle: f32,         // Ángulo actual en la órbita
    orbital_speed: f32,         // Velocidad angular de la órbita
    rotation_speed: f32,        // Velocidad de rotación propia
    scale: f32,                 // Escala del planeta
    planet_type: PlanetType,    // Tipo de shader del planeta
}

// Estructura para el sistema de teletransporte (warping)
struct WarpSystem {
    is_warping: bool,           // Si está en proceso de warp
    warp_progress: f32,          // Progreso del warp (0.0 a 1.0)
    warp_duration: f32,          // Duración total del warp en segundos
    warp_start_time: f32,        // Tiempo cuando comenzó el warp
    target_position: Vector3,    // Posición objetivo de la cámara
    target_look_at: Vector3,     // Punto al que mirar después del warp
    start_position: Vector3,     // Posición inicial de la cámara
    start_look_at: Vector3,       // Punto inicial de mira
}

impl WarpSystem {
    fn new() -> Self {
        WarpSystem {
            is_warping: false,
            warp_progress: 0.0,
            warp_duration: 1.0, // 1 segundo de animación
            warp_start_time: 0.0,
            target_position: Vector3::zero(),
            target_look_at: Vector3::zero(),
            start_position: Vector3::zero(),
            start_look_at: Vector3::zero(),
        }
    }

    fn start_warp(&mut self, current_time: f32, start_pos: Vector3, start_target: Vector3, 
                  target_pos: Vector3, target_look: Vector3) {
        self.is_warping = true;
        self.warp_progress = 0.0;
        self.warp_start_time = current_time;
        self.start_position = start_pos;
        self.start_look_at = start_target;
        self.target_position = target_pos;
        self.target_look_at = target_look;
    }

    fn update(&mut self, current_time: f32) -> bool {
        if !self.is_warping {
            return false;
        }

        let elapsed = current_time - self.warp_start_time;
        self.warp_progress = (elapsed / self.warp_duration).min(1.0);

        if self.warp_progress >= 1.0 {
            self.is_warping = false;
            self.warp_progress = 1.0;
            return true; // Warp completado
        }
        false
    }

    fn get_current_position(&self) -> Vector3 {
        if !self.is_warping {
            return self.target_position;
        }
        // Interpolación suave con easing (ease-in-out)
        let t = self.warp_progress;
        let eased_t = t * t * (3.0 - 2.0 * t); // Smoothstep
        Vector3::new(
            self.start_position.x + (self.target_position.x - self.start_position.x) * eased_t,
            self.start_position.y + (self.target_position.y - self.start_position.y) * eased_t,
            self.start_position.z + (self.target_position.z - self.start_position.z) * eased_t,
        )
    }

    fn get_current_look_at(&self) -> Vector3 {
        if !self.is_warping {
            return self.target_look_at;
        }
        let t = self.warp_progress;
        let eased_t = t * t * (3.0 - 2.0 * t); // Smoothstep
        Vector3::new(
            self.start_look_at.x + (self.target_look_at.x - self.start_look_at.x) * eased_t,
            self.start_look_at.y + (self.target_look_at.y - self.start_look_at.y) * eased_t,
            self.start_look_at.z + (self.target_look_at.z - self.start_look_at.z) * eased_t,
        )
    }
}

fn render(framebuffer: &mut Framebuffer, uniforms: &Uniforms, vertex_array: &[Vertex], light: &Light, planet_type: PlanetType) {
    // Vertex Shader Stage
    let mut transformed_vertices = Vec::with_capacity(vertex_array.len());
    for vertex in vertex_array {
        let transformed = vertex_shader(vertex, uniforms);
        transformed_vertices.push(transformed);
    }

    // Log the first 3 transformed vertices for debugging
    // println!("--- Transformed Vertices (first 3) ---");
    // for i in 0..3.min(transformed_vertices.len()) {
    //     println!("Vertex {}: {:?}", i, transformed_vertices[i].transformed_position);
    // }

    // Primitive Assembly Stage
    let mut triangles = Vec::new();
    for i in (0..transformed_vertices.len()).step_by(3) {
        if i + 2 < transformed_vertices.len() {
            triangles.push([
                transformed_vertices[i].clone(),
                transformed_vertices[i + 1].clone(),
                transformed_vertices[i + 2].clone(),
            ]);
        }
    }

    // Rasterization Stage
    let mut fragments = Vec::new();
    for tri in &triangles {
        fragments.extend(triangle(&tri[0], &tri[1], &tri[2], light));
    }

    // Fragment Processing Stage
    for fragment in fragments {
        // Run fragment shader to compute final color with planet type
        let final_color = fragment_shader_planet(&fragment, uniforms, planet_type);

        framebuffer.point(
            fragment.position.x as i32,
            fragment.position.y as i32,
            final_color,
            fragment.depth
        );
    }
}

/// Función especializada para renderizar el sol con vertex shader especial
fn render_sun(framebuffer: &mut Framebuffer, uniforms: &Uniforms, vertex_array: &[Vertex], light: &Light) {
    // Vertex Shader Stage - Usa el vertex shader especial del sol
    let mut transformed_vertices = Vec::with_capacity(vertex_array.len());
    for vertex in vertex_array {
        let transformed = vertex_shader_sun(vertex, uniforms);
        transformed_vertices.push(transformed);
    }

    // Primitive Assembly Stage
    let mut triangles = Vec::new();
    for i in (0..transformed_vertices.len()).step_by(3) {
        if i + 2 < transformed_vertices.len() {
            triangles.push([
                transformed_vertices[i].clone(),
                transformed_vertices[i + 1].clone(),
                transformed_vertices[i + 2].clone(),
            ]);
        }
    }

    // Rasterization Stage
    let mut fragments = Vec::new();
    for tri in &triangles {
        fragments.extend(triangle(&tri[0], &tri[1], &tri[2], light));
    }

    // Fragment Processing Stage - Usa el shader del sol
    for fragment in fragments {
        let final_color = fragment_shader_planet(&fragment, uniforms, PlanetType::Sun);

        framebuffer.point(
            fragment.position.x as i32,
            fragment.position.y as i32,
            final_color,
            fragment.depth
        );
    }
}

fn main() {
    let window_width = 800;
    let window_height = 600;

    let (mut window, thread) = raylib::init()
        .size(window_width, window_height)
        .title("Solar System")
        .log_level(TraceLogLevel::LOG_WARNING) // Suppress INFO messages
        .build();

    let mut framebuffer = Framebuffer::new(window_width as u32, window_height as u32);
    framebuffer.set_background_color(Vector3::new(0.02, 0.02, 0.05)); // Espacio profundo casi negro

    // Initialize the texture inside the framebuffer
    framebuffer.init_texture(&mut window, &thread);

    // Camera setup - Alejada para ver todo el sistema solar
    // El plano eclíptico está en Y=0, así que la cámara está elevada para ver el plano
    let camera_position = Vector3::new(0.0, 25.0, 50.0);
    let camera_target = Vector3::new(0.0, 0.0, 0.0); // Centro del sistema (donde está el sol)
    let camera_up = Vector3::new(0.0, 1.0, 0.0);
    let mut camera = Camera::new(camera_position, camera_target, camera_up);

    // Projection setup - Aumentar far plane para ver todo el sistema
    let fov_y = PI / 3.0; // 60 degrees
    let aspect = window_width as f32 / window_height as f32;
    let near = 0.1;
    let far = 300.0; // Aumentado para ver planetas más lejanos (órbita más lejana es 36.0)

    // Light setup
    let light = Light::new(Vector3::new(5.0, 5.0, 5.0));

    // Generate sphere mesh programmatically (usaremos el mismo modelo para todos los planetas)
    let sphere = Obj::generate_sphere(1.0, 32); // Radio 1.0, 32 segmentos
    let vertex_array = sphere.get_vertex_array();

    // Crear sistema solar con 5 planetas orbitando
    // Separación aumentada entre planetas y tamaños incrementados
    let mut planets = vec![
        Planet {
            orbital_radius: 12.0,      // Órbita cercana (aumentado de 4.0)
            orbital_angle: 0.0,        // Empieza en ángulo 0
            orbital_speed: 0.5,        // Velocidad rápida
            rotation_speed: 0.05,      // Rotación propia
            scale: 1.5,                 // Planeta pequeño (aumentado de 0.8)
            planet_type: PlanetType::Rocky,
        },
        Planet {
            orbital_radius: 18.0,       // Órbita media (aumentado de 6.0)
            orbital_angle: PI * 2.0 / 5.0, // Empieza a 72 grados
            orbital_speed: 0.3,        // Velocidad media
            rotation_speed: 0.03,
            scale: 2.0,                // Planeta mediano (aumentado de 1.2)
            planet_type: PlanetType::GasGiant,
        },
        Planet {
            orbital_radius: 24.0,       // Órbita lejana (aumentado de 8.0)
            orbital_angle: PI * 4.0 / 5.0, // Empieza a 144 grados
            orbital_speed: 0.2,        // Velocidad lenta
            rotation_speed: 0.02,
            scale: 1.8,                // Planeta normal (aumentado de 1.0)
            planet_type: PlanetType::SciFi,
        },
        Planet {
            orbital_radius: 30.0,      // Órbita muy lejana (aumentado de 10.0)
            orbital_angle: PI * 6.0 / 5.0, // Empieza a 216 grados
            orbital_speed: 0.15,       // Velocidad muy lenta
            rotation_speed: 0.04,
            scale: 1.6,                // Planeta helado (aumentado de 0.9)
            planet_type: PlanetType::Ice,
        },
        Planet {
            orbital_radius: 36.0,      // Órbita más lejana (aumentado de 12.0)
            orbital_angle: PI * 8.0 / 5.0, // Empieza a 288 grados
            orbital_speed: 0.12,       // Velocidad muy lenta
            rotation_speed: 0.06,
            scale: 1.9,                // Planeta volcánico (aumentado de 1.1)
            planet_type: PlanetType::Volcanic,
        },
    ];

    // Generar geometría para anillos (alrededor del gigante gaseoso)
    // Tamaño aumentado proporcionalmente
    let rings = Obj::generate_rings(4.0, 5.5, 16, 32); // Aumentado de 2.5, 3.5
    let rings_vertex_array = rings.get_vertex_array();

    // Generar luna (pequeña esfera que orbita alrededor del planeta rocoso)
    // Tamaño aumentado proporcionalmente
    let moon = Obj::generate_sphere(0.5, 16); // Aumentado de 0.3
    let moon_vertex_array = moon.get_vertex_array();

    // Generar el SOL (esfera en el centro del sistema solar)
    // Usar más segmentos para un sol más suave y detallado
    // Tamaño aumentado para mejor visibilidad
    let sun = Obj::generate_sphere(3.0, 64); // Radio 3.0 (aumentado de 2.0), 64 segmentos para máxima calidad
    let sun_vertex_array = sun.get_vertex_array();

    let mut elapsed_time = 0.0f32;
    let mut warp_system = WarpSystem::new();

    while !window.window_should_close() {
        // Get delta time from Raylib
        let delta_time = window.get_frame_time();
        elapsed_time += delta_time;

        // Manejar teletransporte (warp) con teclas F1-F7
        // F1 = Vista general, F2 = Sol, F3-F7 = Planetas 1-5
        if !warp_system.is_warping {
            for waypoint_idx in 0..7 {
                let key = match waypoint_idx {
                    0 => KeyboardKey::KEY_F1,
                    1 => KeyboardKey::KEY_F2,
                    2 => KeyboardKey::KEY_F3,
                    3 => KeyboardKey::KEY_F4,
                    4 => KeyboardKey::KEY_F5,
                    5 => KeyboardKey::KEY_F6,
                    6 => KeyboardKey::KEY_F7,
                    _ => continue,
                };
                
                if window.is_key_pressed(key) {
                    // Calcular waypoint basado en las posiciones actuales de los planetas
                    let (target_pos, target_look) = match waypoint_idx {
                        0 => {
                            // Vista general del sistema
                            (Vector3::new(0.0, 40.0, 60.0), Vector3::new(0.0, 0.0, 0.0))
                        },
                        1 => {
                            // Cerca del Sol
                            (Vector3::new(0.0, 8.0, 12.0), Vector3::new(0.0, 0.0, 0.0))
                        },
                        i if i >= 2 && i <= 6 => {
                            // Waypoints 2-6: Cada planeta
                            let planet_idx = i - 2;
                            if planet_idx < planets.len() {
                                let planet = &planets[planet_idx];
                                let orbit_x = planet.orbital_radius * planet.orbital_angle.cos();
                                let orbit_z = planet.orbital_radius * planet.orbital_angle.sin();
                                let planet_pos = Vector3::new(orbit_x, 0.0, orbit_z);
                                
                                let offset_distance = 8.0 + planet.scale * 2.0;
                                let camera_pos = Vector3::new(
                                    orbit_x + offset_distance * 0.7,
                                    5.0 + planet.scale,
                                    orbit_z + offset_distance * 0.7,
                                );
                                (camera_pos, planet_pos)
                            } else {
                                (Vector3::new(0.0, 25.0, 50.0), Vector3::new(0.0, 0.0, 0.0))
                            }
                        },
                        _ => (Vector3::new(0.0, 40.0, 60.0), Vector3::new(0.0, 0.0, 0.0)),
                    };
                    
                    warp_system.start_warp(
                        elapsed_time,
                        camera.eye,
                        camera.target,
                        target_pos,
                        target_look,
                    );
                    // Desactivar seguimiento de planeta durante el warp
                    camera.track_planet(None);
                    break;
                }
            }
        }

        // Actualizar sistema de warping
        let warp_completed = warp_system.update(elapsed_time);
        if warp_completed {
            // Warp completado, actualizar cámara a la posición final
            camera.warp_to(warp_system.get_current_position(), warp_system.get_current_look_at());
        } else if warp_system.is_warping {
            // Durante el warp, interpolar la posición de la cámara de forma optimizada
            // Actualizar directamente las posiciones (más eficiente que warp_to completo)
            let current_pos = warp_system.get_current_position();
            let current_target = warp_system.get_current_look_at();
            
            // Actualizar directamente sin cálculos trigonométricos costosos
            camera.eye = current_pos;
            camera.target = current_target;
            camera.ecliptic_height = current_pos.y;
            
            // Calcular distancia de forma simple (sin trigonometría innecesaria)
            let dx = current_pos.x - current_target.x;
            let dy = current_pos.y - current_target.y;
            let dz = current_pos.z - current_target.z;
            camera.distance = (dx * dx + dy * dy + dz * dz).sqrt();
            // No necesitamos actualizar ángulos durante el warp, la posición directa es suficiente
        }

        // Manejar cambio de planeta con teclas numéricas
        // 0 = Sol, 1-5 = Planetas (solo si no está en warp)
        if !warp_system.is_warping {
            if window.is_key_pressed(KeyboardKey::KEY_ZERO) || window.is_key_pressed(KeyboardKey::KEY_KP_0) {
                camera.track_planet(None); // Seguir el sol (centro)
            }
            for i in 0..planets.len() {
                let key = match i {
                    0 => KeyboardKey::KEY_ONE,
                    1 => KeyboardKey::KEY_TWO,
                    2 => KeyboardKey::KEY_THREE,
                    3 => KeyboardKey::KEY_FOUR,
                    4 => KeyboardKey::KEY_FIVE,
                    _ => continue,
                };
                let key_numpad = match i {
                    0 => KeyboardKey::KEY_KP_1,
                    1 => KeyboardKey::KEY_KP_2,
                    2 => KeyboardKey::KEY_KP_3,
                    3 => KeyboardKey::KEY_KP_4,
                    4 => KeyboardKey::KEY_KP_5,
                    _ => continue,
                };
                if window.is_key_pressed(key) || window.is_key_pressed(key_numpad) {
                    camera.track_planet(Some(i));
                }
            }
        }

        // Process camera input (deshabilitado durante warp)
        if !warp_system.is_warping {
            camera.process_input(&window);
        }

        // Update orbital positions and rotations
        for planet in &mut planets {
            planet.orbital_angle += planet.orbital_speed * delta_time;
            if planet.orbital_angle >= 2.0 * PI {
                planet.orbital_angle -= 2.0 * PI;
            }
        }

        framebuffer.clear();

        let view_matrix = camera.get_view_matrix();
        let projection_matrix = create_projection_matrix(fov_y, aspect, near, far);
        let viewport_matrix = create_viewport_matrix(0.0, 0.0, window_width as f32, window_height as f32);

        // ======================================
        // RENDERIZAR EL SOL EN EL CENTRO
        // ======================================
        let sun_translation = Vector3::new(0.0, 0.0, 0.0); // Centro del sistema
        let sun_rotation = Vector3::new(0.0, elapsed_time * 0.1, 0.0); // Rotación lenta del sol
        let sun_model_matrix = create_model_matrix(sun_translation, 1.0, sun_rotation);
        
        let sun_uniforms = Uniforms {
            model_matrix: sun_model_matrix,
            view_matrix,
            projection_matrix,
            viewport_matrix,
            time: elapsed_time,
        };

        // Usar la función especializada render_sun
        render_sun(&mut framebuffer, &sun_uniforms, &sun_vertex_array, &light);

        // Actualizar seguimiento del sol si la cámara está en modo libre (None)
        if camera.get_tracking_planet().is_none() {
            camera.update_planet_tracking(sun_translation); // Seguir el sol (centro)
        }

        // Renderizar cada planeta en su órbita
        for (idx, planet) in planets.iter().enumerate() {
            // Calcular posición orbital en el plano eclíptico (XZ, Y=0)
            let orbit_x = planet.orbital_radius * planet.orbital_angle.cos();
            let orbit_z = planet.orbital_radius * planet.orbital_angle.sin();
            let orbit_y = 0.0; // Todos en el mismo plano eclíptico (Y=0)
            
            let translation = Vector3::new(orbit_x, orbit_y, orbit_z);

            // Actualizar seguimiento de planeta si la cámara está siguiendo este planeta
            if camera.get_tracking_planet() == Some(idx) {
                camera.update_planet_tracking(translation);
            }
            
            // Rotación propia del planeta alrededor de su eje Y
            let planet_self_rotation = elapsed_time * planet.rotation_speed;
            let rotation = Vector3::new(0.0, planet_self_rotation, 0.0);
            
            let model_matrix = create_model_matrix(translation, planet.scale, rotation);
            
            let uniforms = Uniforms {
                model_matrix,
                view_matrix,
                projection_matrix,
                viewport_matrix,
                time: elapsed_time,
            };

            render(&mut framebuffer, &uniforms, &vertex_array, &light, planet.planet_type);

            // Renderizar anillos alrededor del gigante gaseoso (índice 1)
            if idx == 1 {
                // Anillos están pegados al planeta y rotan con él
                // Usar la misma rotación que el planeta para que giren juntos
                let rings_matrix = create_model_matrix(translation, 1.0, rotation);
                let rings_uniforms = Uniforms {
                    model_matrix: rings_matrix,
                    view_matrix,
                    projection_matrix,
                    viewport_matrix,
                    time: elapsed_time,
                };
                render(&mut framebuffer, &rings_uniforms, &rings_vertex_array, &light, PlanetType::Ring);
            }

            // Renderizar luna orbitando alrededor del primer planeta (índice 0)
            if idx == 0 {
                // La luna orbita alrededor del planeta rocoso
                // Radio orbital aumentado proporcionalmente al nuevo tamaño del planeta
                let moon_orbital_radius = 2.5; // Aumentado de 1.5
                let moon_orbital_angle = elapsed_time * 1.0; // Velocidad orbital de la luna
                let moon_orbit_x = orbit_x + moon_orbital_radius * moon_orbital_angle.cos();
                let moon_orbit_z = orbit_z + moon_orbital_radius * moon_orbital_angle.sin();
                let moon_orbit_y = 0.3; // Ligeramente elevada (aumentado proporcionalmente)
                
                let moon_translation = Vector3::new(moon_orbit_x, moon_orbit_y, moon_orbit_z);
                let moon_rotation = Vector3::new(0.0, elapsed_time * 0.1, 0.0);
                let moon_matrix = create_model_matrix(moon_translation, 1.0, moon_rotation);
                
                let moon_uniforms = Uniforms {
                    model_matrix: moon_matrix,
                    view_matrix,
                    projection_matrix,
                    viewport_matrix,
                    time: elapsed_time,
                };
                render(&mut framebuffer, &moon_uniforms, &moon_vertex_array, &light, PlanetType::Moon);
            }
        }

        // Actualizar textura del framebuffer y dibujar todo en un solo frame
        framebuffer.update_texture();

        let mut d = window.begin_drawing(&thread);
        d.clear_background(Color::BLACK);
        framebuffer.draw_to(&mut d);

        // Crosshair centrado
        let center_x = window_width / 2;
        let center_y = window_height / 2;
        let crosshair_size = 10;
        d.draw_line(center_x - crosshair_size, center_y, center_x + crosshair_size, center_y, Color::WHITE);
        d.draw_line(center_x, center_y - crosshair_size, center_x, center_y + crosshair_size, Color::WHITE);

        // Efecto visual de warp (teletransporte animado) - OPTIMIZADO
        if warp_system.is_warping {
            let progress = warp_system.warp_progress;
            
            // Calcular valores una sola vez
            let max_radius = ((window_width * window_width + window_height * window_height) as f32).sqrt() * 0.5;
            let current_radius = max_radius * progress;
            let alpha_factor = (1.0 - (progress - 0.5).abs() * 2.0).max(0.0);
            
            // Efecto simplificado: solo líneas radiales esenciales (reducido de 20 a 8)
            let num_lines = 8;
            let warp_intensity = (progress * PI * 2.0).sin() * 0.5 + 0.5;
            let warp_alpha = (200.0 * alpha_factor) as u8;
            let warp_color = Color::new(
                (50.0 + warp_intensity * 100.0) as u8,
                (150.0 + warp_intensity * 105.0) as u8,
                255,
                warp_alpha,
            );
            
            // Pre-calcular valores constantes
            let angle_step = PI * 2.0 / num_lines as f32;
            let time_factor = elapsed_time * 5.0;
            
            // Dibujar líneas radiales optimizadas
            for i in 0..num_lines {
                let angle = i as f32 * angle_step;
                let cos_angle = angle.cos();
                let sin_angle = angle.sin();
                
                // Longitud variable simplificada
                let line_length = current_radius * (0.8 + 0.2 * (time_factor + angle).sin());
                let line_end_x = center_x as f32 + line_length * cos_angle;
                let line_end_y = center_y as f32 + line_length * sin_angle;
                
                d.draw_line(
                    center_x,
                    center_y,
                    line_end_x as i32,
                    line_end_y as i32,
                    warp_color,
                );
            }
            
            // Solo 2 círculos concéntricos en lugar de 5
            if current_radius > 10.0 {
                let circle_alpha = (150.0 * alpha_factor) as u8;
                let circle_color = Color::new(100, 150, 255, circle_alpha);
                d.draw_circle_lines(center_x, center_y, current_radius * 0.5, circle_color);
                d.draw_circle_lines(center_x, center_y, current_radius, circle_color);
            }
        }

        // Control de FPS optimizado - solo sleep si el frame fue muy rápido
        // Esto permite mejor rendimiento durante warp
        let frame_time_ms = delta_time * 1000.0;
        if frame_time_ms < 16.0 && !warp_system.is_warping {
            thread::sleep(Duration::from_millis((16.0 - frame_time_ms) as u64));
        }
    }
}
