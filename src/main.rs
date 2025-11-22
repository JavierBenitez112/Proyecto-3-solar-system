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

// Estructura para la nave espacial
struct Ship {
    position: Vector3,           // Posición de la nave
    rotation: Vector3,           // Rotación de la nave (yaw, pitch, roll)
    velocity: Vector3,           // Velocidad de la nave
    #[allow(dead_code)]
    speed: f32,                  // Velocidad máxima (no se usa actualmente, la nave sigue a la cámara)
    #[allow(dead_code)]
    rotation_speed: f32,        // Velocidad de rotación (no se usa actualmente, la nave sigue a la cámara)
    scale: f32,                  // Escala del modelo
    use_direct_rotation: bool,   // Si es true, usa rotación directa; si es false, sigue a la cámara
}

impl Ship {
    fn new() -> Self {
        Ship {
            position: Vector3::new(0.0, 20.0, 40.0), // Posición inicial delante de la cámara
            rotation: Vector3::new(0.0, 0.0, 0.0),  // Sin rotación inicial
            velocity: Vector3::zero(),
            speed: 25.0,          // Velocidad de movimiento con flechas (aumentada)
            rotation_speed: 1.0,  // Velocidad de rotación con WASD (disminuida)
            scale: 0.5,           // Escala para la nave (ajustada para mejor visibilidad)
            use_direct_rotation: false, // Por defecto, la nave sigue a la cámara
        }
    }

    fn update(&mut self, delta_time: f32) {
        // Aplicar velocidad a la posición
        self.position.x += self.velocity.x * delta_time;
        self.position.y += self.velocity.y * delta_time;
        self.position.z += self.velocity.z * delta_time;
        
        // Aplicar fricción a la velocidad
        self.velocity.x *= 0.95;
        self.velocity.y *= 0.95;
        self.velocity.z *= 0.95;
    }

    /// Establece la rotación de la nave directamente en valores absolutos (en radianes)
    /// 
    /// # Argumentos
    /// * `pitch` - Rotación alrededor del eje X (pitch) en radianes
    /// * `yaw` - Rotación alrededor del eje Y (yaw) en radianes
    /// * `roll` - Rotación alrededor del eje Z (roll) en radianes
    /// 
    /// # Ejemplo
    /// ```
    /// use std::f32::consts::PI;
    /// ship.set_rotation(0.5, 1.0, 0.0); // Pitch: 0.5 rad, Yaw: 1.0 rad, Roll: 0.0 rad
    /// ship.set_rotation(45.0 * PI / 180.0, 90.0 * PI / 180.0, 0.0); // Usando grados convertidos
    /// ```
    pub fn set_rotation(&mut self, pitch: f32, yaw: f32, roll: f32) {
        self.rotation.x = pitch;
        self.rotation.y = yaw;
        self.rotation.z = roll;
    }

    /// Establece solo el pitch (rotación X) de la nave
    pub fn set_pitch(&mut self, pitch: f32) {
        self.rotation.x = pitch;
    }

    /// Establece solo el yaw (rotación Y) de la nave
    pub fn set_yaw(&mut self, yaw: f32) {
        self.rotation.y = yaw;
    }

    /// Establece solo el roll (rotación Z) de la nave
    pub fn set_roll(&mut self, roll: f32) {
        self.rotation.z = roll;
    }

    /// Obtiene la rotación actual de la nave
    pub fn get_rotation(&self) -> Vector3 {
        self.rotation
    }

    /// Habilita o deshabilita la rotación directa
    /// Si `enabled` es true, la nave usará rotación directa (no seguirá a la cámara)
    /// Si `enabled` es false, la nave seguirá la rotación de la cámara
    pub fn set_direct_rotation(&mut self, enabled: bool) {
        self.use_direct_rotation = enabled;
    }

    // ======================================
    // MÉTODOS PARA ROTAR EL MODELO 3D DIRECTAMENTE
    // ======================================

    /// Rota el modelo alrededor del eje X (pitch) agregando rotación
    /// 
    /// # Argumentos
    /// * `angle` - Ángulo en radianes a agregar a la rotación actual
    /// 
    /// # Ejemplo
    /// ```
    /// ship.rotate_pitch(0.5); // Rota 0.1 radianes alrededor del eje X
    /// ```
    pub fn rotate_pitch(&mut self, angle: f32) {
        self.rotation.x += angle;
    }

    /// Rota el modelo alrededor del eje Y (yaw) agregando rotación
    /// 
    /// # Argumentos
    /// * `angle` - Ángulo en radianes a agregar a la rotación actual
    /// 
    /// # Ejemplo
    /// ```
    /// ship.rotate_yaw(0.6); // Rota 0.1 radianes alrededor del eje Y
    /// ```
    pub fn rotate_yaw(&mut self, angle: f32) {
        self.rotation.y += angle;
    }

    /// Rota el modelo alrededor del eje Z (roll) agregando rotación
    /// 
    /// # Argumentos
    /// * `angle` - Ángulo en radianes a agregar a la rotación actual
    /// 
    /// # Ejemplo
    /// ```
    /// ship.rotate_roll(0.3); // Rota 0.1 radianes alrededor del eje Z
    /// ```
    pub fn rotate_roll(&mut self, angle: f32) {
        self.rotation.z += angle;
    }

    /// Rota el modelo en todos los ejes agregando rotación
    /// 
    /// # Argumentos
    /// * `pitch` - Rotación a agregar en X (radianes)
    /// * `yaw` - Rotación a agregar en Y (radianes)
    /// * `roll` - Rotación a agregar en Z (radianes)
    /// 
    /// # Ejemplo
    /// ```
    /// ship.rotate(0.1, 0.2, 0.05); // Rota en todos los ejes
    /// ```
    pub fn rotate(&mut self, pitch: f32, yaw: f32, roll: f32) {
        self.rotation.x += pitch;
        self.rotation.y += yaw;
        self.rotation.z += roll;
    }

    /// Rota el modelo alrededor del eje X usando grados
    /// 
    /// # Argumentos
    /// * `degrees` - Ángulo en grados a agregar a la rotación actual
    /// 
    /// # Ejemplo
    /// ```
    /// ship.rotate_pitch_degrees(45.0); // Rota 45 grados alrededor del eje X
    /// ```
    pub fn rotate_pitch_degrees(&mut self, degrees: f32) {
        use std::f32::consts::PI;
        self.rotation.x += degrees * PI / 180.0;
    }

    /// Rota el modelo alrededor del eje Y usando grados
    /// 
    /// # Argumentos
    /// * `degrees` - Ángulo en grados a agregar a la rotación actual
    /// 
    /// # Ejemplo
    /// ```
    /// ship.rotate_yaw_degrees(90.0); // Rota 90 grados alrededor del eje Y
    /// ```
    pub fn rotate_yaw_degrees(&mut self, degrees: f32) {
        use std::f32::consts::PI;
        self.rotation.y += degrees * PI / 180.0;
    }

    /// Rota el modelo alrededor del eje Z usando grados
    /// 
    /// # Argumentos
    /// * `degrees` - Ángulo en grados a agregar a la rotación actual
    /// 
    /// # Ejemplo
    /// ```
    /// ship.rotate_roll_degrees(180.0); // Rota 180 grados alrededor del eje Z
    /// ```
    pub fn rotate_roll_degrees(&mut self, degrees: f32) {
        use std::f32::consts::PI;
        self.rotation.z += degrees * PI / 180.0;
    }

    /// Rota el modelo en todos los ejes usando grados
    /// 
    /// # Argumentos
    /// * `pitch_degrees` - Rotación a agregar en X (grados)
    /// * `yaw_degrees` - Rotación a agregar en Y (grados)
    /// * `roll_degrees` - Rotación a agregar en Z (grados)
    /// 
    /// # Ejemplo
    /// ```
    /// ship.rotate_degrees(45.0, 90.0, 0.0); // Rota usando grados
    /// ```
    pub fn rotate_degrees(&mut self, pitch_degrees: f32, yaw_degrees: f32, roll_degrees: f32) {
        use std::f32::consts::PI;
        self.rotation.x += pitch_degrees * PI / 180.0;
        self.rotation.y += yaw_degrees * PI / 180.0;
        self.rotation.z += roll_degrees * PI / 180.0;
    }

    /// Establece la rotación usando grados (más intuitivo que radianes)
    /// 
    /// # Argumentos
    /// * `pitch_degrees` - Pitch en grados
    /// * `yaw_degrees` - Yaw en grados
    /// * `roll_degrees` - Roll en grados
    /// 
    /// # Ejemplo
    /// ```
    /// ship.set_rotation_degrees(45.0, 90.0, 0.0); // Establece rotación usando grados
    /// ```
    pub fn set_rotation_degrees(&mut self, pitch_degrees: f32, yaw_degrees: f32, roll_degrees: f32) {
        use std::f32::consts::PI;
        self.rotation.x = pitch_degrees * PI / 180.0;
        self.rotation.y = yaw_degrees * PI / 180.0;
        self.rotation.z = roll_degrees * PI / 180.0;
    }

    /// Rota el modelo continuamente a una velocidad específica
    /// Útil para animaciones o rotación automática
    /// 
    /// # Argumentos
    /// * `pitch_speed` - Velocidad de rotación en X (radianes por segundo)
    /// * `yaw_speed` - Velocidad de rotación en Y (radianes por segundo)
    /// * `roll_speed` - Velocidad de rotación en Z (radianes por segundo)
    /// * `delta_time` - Tiempo transcurrido desde el último frame
    /// 
    /// # Ejemplo
    /// ```
    /// ship.rotate_continuous(0.0, 1.0, 0.0, delta_time); // Rota continuamente en Y
    /// ```
    pub fn rotate_continuous(&mut self, pitch_speed: f32, yaw_speed: f32, roll_speed: f32, delta_time: f32) {
        self.rotation.x += pitch_speed * delta_time;
        self.rotation.y += yaw_speed * delta_time;
        self.rotation.z += roll_speed * delta_time;
    }

    /// Rota el modelo continuamente usando grados por segundo
    /// 
    /// # Argumentos
    /// * `pitch_degrees_per_sec` - Velocidad de rotación en X (grados por segundo)
    /// * `yaw_degrees_per_sec` - Velocidad de rotación en Y (grados por segundo)
    /// * `roll_degrees_per_sec` - Velocidad de rotación en Z (grados por segundo)
    /// * `delta_time` - Tiempo transcurrido desde el último frame
    /// 
    /// # Ejemplo
    /// ```
    /// ship.rotate_continuous_degrees(0.0, 90.0, 0.0, delta_time); // Rota 90 grados/seg en Y
    /// ```
    pub fn rotate_continuous_degrees(&mut self, pitch_degrees_per_sec: f32, yaw_degrees_per_sec: f32, roll_degrees_per_sec: f32, delta_time: f32) {
        use std::f32::consts::PI;
        self.rotation.x += pitch_degrees_per_sec * PI / 180.0 * delta_time;
        self.rotation.y += yaw_degrees_per_sec * PI / 180.0 * delta_time;
        self.rotation.z += roll_degrees_per_sec * PI / 180.0 * delta_time;
    }

    /// Resetea la rotación del modelo a cero
    /// 
    /// # Ejemplo
    /// ```
    /// ship.reset_rotation(); // Vuelve la rotación a (0, 0, 0)
    /// ```
    pub fn reset_rotation(&mut self) {
        self.rotation.x = 0.0;
        self.rotation.y = 0.0;
        self.rotation.z = 0.0;
    }

    #[allow(dead_code)]
    fn process_input(&mut self, window: &RaylibHandle, delta_time: f32) {
        // Rotación con Q y E (roll)
        if window.is_key_down(KeyboardKey::KEY_Q) {
            self.rotation.z += self.rotation_speed * delta_time;
        }
        if window.is_key_down(KeyboardKey::KEY_E) {
            self.rotation.z -= self.rotation_speed * delta_time;
        }

        // Rotación con A y D (yaw)
        if window.is_key_down(KeyboardKey::KEY_A) {
            self.rotation.y += self.rotation_speed * delta_time;
        }
        if window.is_key_down(KeyboardKey::KEY_D) {
            self.rotation.y -= self.rotation_speed * delta_time;
        }

        // Rotación con W y S (pitch)
        if window.is_key_down(KeyboardKey::KEY_W) {
            self.rotation.x += self.rotation_speed * delta_time;
        }
        if window.is_key_down(KeyboardKey::KEY_S) {
            self.rotation.x -= self.rotation_speed * delta_time;
        }

        // Movimiento con flechas - más directo y responsivo
        // Calcular dirección forward basada en la rotación completa (yaw y pitch)
        let cos_y = self.rotation.y.cos();
        let sin_y = self.rotation.y.sin();
        let cos_x = self.rotation.x.cos();
        let sin_x = self.rotation.x.sin();
        
        let forward = Vector3::new(
            sin_y * cos_x,
            -sin_x,
            cos_y * cos_x,
        );
        
        // Movimiento hacia adelante/atrás con flechas arriba/abajo
        if window.is_key_down(KeyboardKey::KEY_UP) {
            self.velocity.x += forward.x * self.speed * delta_time;
            self.velocity.y += forward.y * self.speed * delta_time;
            self.velocity.z += forward.z * self.speed * delta_time;
        }
        if window.is_key_down(KeyboardKey::KEY_DOWN) {
            self.velocity.x -= forward.x * self.speed * delta_time;
            self.velocity.y -= forward.y * self.speed * delta_time;
            self.velocity.z -= forward.z * self.speed * delta_time;
        }

        // Movimiento lateral con flechas izquierda/derecha
        let right = Vector3::new(
            (self.rotation.y + PI / 2.0).sin() * cos_x,
            0.0,
            (self.rotation.y + PI / 2.0).cos() * cos_x,
        );
        
        if window.is_key_down(KeyboardKey::KEY_LEFT) {
            self.velocity.x += right.x * self.speed * delta_time;
            self.velocity.z += right.z * self.speed * delta_time;
        }
        if window.is_key_down(KeyboardKey::KEY_RIGHT) {
            self.velocity.x -= right.x * self.speed * delta_time;
            self.velocity.z -= right.z * self.speed * delta_time;
        }

        // Movimiento vertical con R y F
        if window.is_key_down(KeyboardKey::KEY_R) {
            self.velocity.y += self.speed * delta_time;
        }
        if window.is_key_down(KeyboardKey::KEY_F) {
            self.velocity.y -= self.speed * delta_time;
        }
    }
    
    // Obtener la dirección forward de la nave (para la cámara)
    #[allow(dead_code)]
    fn get_forward_direction(&self) -> Vector3 {
        let cos_y = self.rotation.y.cos();
        let sin_y = self.rotation.y.sin();
        let cos_x = self.rotation.x.cos();
        let sin_x = self.rotation.x.sin();
        
        Vector3::new(
            sin_y * cos_x,
            -sin_x,
            cos_y * cos_x,
        )
    }
    
    // Obtener la dirección up de la nave (para la cámara)
    #[allow(dead_code)]
    fn get_up_direction(&self) -> Vector3 {
        let cos_y = self.rotation.y.cos();
        let sin_y = self.rotation.y.sin();
        let cos_x = self.rotation.x.cos();
        let sin_x = self.rotation.x.sin();
        let cos_z = self.rotation.z.cos();
        let sin_z = self.rotation.z.sin();
        
        // Calcular up basado en roll
        let up_local = Vector3::new(
            sin_y * sin_x * cos_z - cos_y * sin_z,
            cos_x * cos_z,
            cos_y * sin_x * cos_z + sin_y * sin_z,
        );
        up_local
    }
}

// Estructura para el sistema de teletransporte (warping) - ahora sobre la nave y la cámara
struct WarpSystem {
    is_warping: bool,           // Si está en proceso de warp
    warp_progress: f32,          // Progreso del warp (0.0 a 1.0)
    warp_duration: f32,          // Duración total del warp en segundos
    warp_start_time: f32,        // Tiempo cuando comenzó el warp
    target_ship_position: Vector3,    // Posición objetivo de la nave
    start_ship_position: Vector3,     // Posición inicial de la nave
    target_camera_position: Vector3,  // Posición objetivo de la cámara
    start_camera_position: Vector3,    // Posición inicial de la cámara
}

impl WarpSystem {
    fn new() -> Self {
        WarpSystem {
            is_warping: false,
            warp_progress: 0.0,
            warp_duration: 1.0, // 1 segundo de animación
            warp_start_time: 0.0,
            target_ship_position: Vector3::zero(),
            start_ship_position: Vector3::zero(),
            target_camera_position: Vector3::zero(),
            start_camera_position: Vector3::zero(),
        }
    }

    fn start_warp(&mut self, current_time: f32, start_ship_pos: Vector3, target_ship_pos: Vector3, start_camera_pos: Vector3, target_camera_pos: Vector3) {
        self.is_warping = true;
        self.warp_progress = 0.0;
        self.warp_start_time = current_time;
        self.start_ship_position = start_ship_pos;
        self.target_ship_position = target_ship_pos;
        self.start_camera_position = start_camera_pos;
        self.target_camera_position = target_camera_pos;
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

    fn get_current_ship_position(&self) -> Vector3 {
        if !self.is_warping {
            return self.target_ship_position;
        }
        // Interpolación suave con easing (ease-in-out)
        let t = self.warp_progress;
        let eased_t = t * t * (3.0 - 2.0 * t); // Smoothstep
        Vector3::new(
            self.start_ship_position.x + (self.target_ship_position.x - self.start_ship_position.x) * eased_t,
            self.start_ship_position.y + (self.target_ship_position.y - self.start_ship_position.y) * eased_t,
            self.start_ship_position.z + (self.target_ship_position.z - self.start_ship_position.z) * eased_t,
        )
    }

    fn get_current_camera_position(&self) -> Vector3 {
        if !self.is_warping {
            return self.target_camera_position;
        }
        // Interpolación suave con easing (ease-in-out)
        let t = self.warp_progress;
        let eased_t = t * t * (3.0 - 2.0 * t); // Smoothstep
        Vector3::new(
            self.start_camera_position.x + (self.target_camera_position.x - self.start_camera_position.x) * eased_t,
            self.start_camera_position.y + (self.target_camera_position.y - self.start_camera_position.y) * eased_t,
            self.start_camera_position.z + (self.target_camera_position.z - self.start_camera_position.z) * eased_t,
        )
    }
}

fn render(framebuffer: &mut Framebuffer, uniforms: &Uniforms, vertex_array: &[Vertex], light: &Light, planet_type: PlanetType) {
    // Optimización: Early exit si el array está vacío
    if vertex_array.is_empty() {
        return;
    }
    
    // Optimización: Límite de vértices para modelos muy grandes (solo para la nave)
    const MAX_VERTICES: usize = 100000; // Aumentado para modelos grandes
    let effective_array = if vertex_array.len() > MAX_VERTICES {
        // Para modelos muy grandes, usar solo los primeros MAX_VERTICES
        &vertex_array[..MAX_VERTICES]
    } else {
        vertex_array
    };
    
    // Vertex Shader Stage
    let mut transformed_vertices = Vec::with_capacity(effective_array.len());
    for vertex in effective_array {
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
    framebuffer.set_background_color(Vector3::new(0.0, 0.0, 0.0)); // Fondo negro para el espacio

    // Initialize the texture inside the framebuffer
    framebuffer.init_texture(&mut window, &thread);

    // Generar estrellas para el skybox
    // Usar una semilla fija para que las estrellas sean consistentes
    let num_stars = 2000; // Número de estrellas
    let mut stars = Vec::new();
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    "star_seed".hash(&mut hasher);
    let seed = hasher.finish();
    
    // Generar posiciones de estrellas usando un generador pseudoaleatorio simple
    let mut rng_state = seed;
    for _ in 0..num_stars {
        // Generador LCG simple
        rng_state = rng_state.wrapping_mul(1103515245).wrapping_add(12345);
        let x = (rng_state % window_width as u64) as i32;
        rng_state = rng_state.wrapping_mul(1103515245).wrapping_add(12345);
        let y = (rng_state % window_height as u64) as i32;
        rng_state = rng_state.wrapping_mul(1103515245).wrapping_add(12345);
        // Variar el brillo de las estrellas (0.5 a 1.0)
        let brightness = 0.5 + ((rng_state % 50) as f32 / 100.0);
        stars.push((x, y, brightness));
    }

    // Inicializar la nave
    let mut ship = Ship::new();
    
    // Camera setup - Cámara libre con zoom fijo
    // Posición inicial donde la nave estará visible delante de la cámara
    let camera_initial_position = Vector3::new(0.0, 20.0, 60.0); // Posición inicial de la cámara
    let camera_initial_target = Vector3::new(0.0, 20.0, 40.0); // Target inicial (nave estará aquí)
    let camera_up = Vector3::new(0.0, 1.0, 0.0);
    let mut camera = Camera::new(camera_initial_position, camera_initial_target, camera_up);
    
    // Fijar la distancia (zoom fijo)
    camera.distance = 20.0; // Distancia fija para vista tercera persona

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

    // Cargar el modelo 3D de la nave (Untitled.obj)
    println!("Cargando modelo 3D de la nave...");
    let ship_model = match Obj::load("assets/models/Untitled.obj") {
        Ok(model) => {
            let vertex_count = model.get_vertex_array().len();
            println!("Modelo de nave cargado: {} vértices", vertex_count);
            model
        },
        Err(e) => {
            eprintln!("Error al cargar Untitled.obj: {:?}. Usando esfera como fallback.", e);
            // Si no se puede cargar, usar una esfera como placeholder
            Obj::generate_sphere(1.0, 16)
        }
    };
    
    // Pre-cargar el vertex array de la nave
    let ship_vertex_array = ship_model.get_vertex_array();
    println!("Nave lista para renderizar con {} vértices", ship_vertex_array.len());

    let mut elapsed_time = 0.0f32;
    let mut warp_system = WarpSystem::new();

    while !window.window_should_close() {
        // Get delta time from Raylib
        let delta_time = window.get_frame_time();
        elapsed_time += delta_time;

        // Procesar entrada de la cámara (la nave seguirá a la cámara)
        // Deshabilitar input durante el warp para evitar interferencias
        if !warp_system.is_warping {
            camera.process_input(&window);
        }
        
        // La nave ya no procesa input directamente, sigue a la cámara
        // ship.process_input(&window, delta_time); // Deshabilitado - la nave sigue a la cámara
        ship.update(delta_time);

        // ======================================
        // EJEMPLO: Rotar el modelo 3D directamente por código
        // ======================================
        // Para rotar el modelo, descomenta las siguientes líneas:
        //
        // // 1. Habilitar rotación directa (desactiva la sincronización con la cámara)
        // ship.set_direct_rotation(true);
        //
        // // 2. ROTAR EL MODELO - Métodos más directos:
        //
        // // Rotar agregando rotación (más común):
        // ship.rotate_pitch(0.1);        // Rota 0.1 radianes en X
        // ship.rotate_yaw(0.2);          // Rota 0.2 radianes en Y
        // ship.rotate_roll(0.05);        // Rota 0.05 radianes en Z
        //
        // // O rotar todos los ejes a la vez:
        // ship.rotate(0.1, 0.2, 0.05);   // Rota en todos los ejes
        //
        // // Rotar usando grados (más intuitivo):
        // ship.rotate_pitch_degrees(45.0);  // Rota 45 grados en X
        // ship.rotate_yaw_degrees(90.0);    // Rota 90 grados en Y
        // ship.rotate_roll_degrees(180.0);  // Rota 180 grados en Z
        //
        // // O rotar todos los ejes usando grados:
        // ship.rotate_degrees(45.0, 90.0, 0.0);
        //
        // // Rotación continua (útil para animaciones):
        // ship.rotate_continuous(0.0, 1.0, 0.0, delta_time); // Rota 1 rad/seg en Y
        // ship.rotate_continuous_degrees(0.0, 90.0, 0.0, delta_time); // Rota 90°/seg en Y
        //
        // // Establecer rotación absoluta:
        // ship.set_rotation(0.5, 1.0, 0.0);              // En radianes
        // ship.set_rotation_degrees(45.0, 90.0, 0.0);    // En grados
        //
        // // Resetear rotación:
        // ship.reset_rotation(); // Vuelve a (0, 0, 0)
        //
        // // Para volver a la sincronización con la cámara:
        // ship.set_direct_rotation(false);
        // ======================================

        // Manejar teletransporte (warp) con teclas F1-F7 - ahora sobre la nave
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
                    let target_pos = match waypoint_idx {
                        0 => {
                            // Vista general del sistema
                            Vector3::new(0.0, 40.0, 60.0)
                        },
                        1 => {
                            // Cerca del Sol
                            Vector3::new(0.0, 8.0, 12.0)
                        },
                        i if i >= 2 && i <= 6 => {
                            // Waypoints 2-6: Cada planeta
                            let planet_idx = i - 2;
                            if planet_idx < planets.len() {
                                let planet = &planets[planet_idx];
                                let orbit_x = planet.orbital_radius * planet.orbital_angle.cos();
                                let orbit_z = planet.orbital_radius * planet.orbital_angle.sin();
                                Vector3::new(orbit_x, 5.0, orbit_z)
                            } else {
                                Vector3::new(0.0, 25.0, 50.0)
                            }
                        },
                        _ => Vector3::new(0.0, 40.0, 60.0),
                    };
                    
                    // Calcular posición objetivo de la cámara basándose en la posición objetivo de la nave
                    // Mantener el offset relativo entre la cámara y la nave
                    let ship_offset_forward = 20.0; // Distancia fija delante de la cámara
                    let ship_offset_down = -2.0; // Ligeramente abajo
                    
                    // Calcular dirección forward de la cámara actual para mantener la orientación
                    let cos_yaw = camera.yaw.cos();
                    let sin_yaw = camera.yaw.sin();
                    let cos_pitch = camera.pitch.cos();
                    let sin_pitch = camera.pitch.sin();
                    
                    let camera_forward = Vector3::new(
                        cos_yaw * cos_pitch,
                        sin_pitch,
                        sin_yaw * cos_pitch,
                    );
                    
                    let _camera_right = Vector3::new(
                        -sin_yaw,
                        0.0,
                        cos_yaw,
                    );
                    
                    let camera_up_dir = Vector3::new(
                        -cos_yaw * sin_pitch,
                        cos_pitch,
                        -sin_yaw * sin_pitch,
                    );
                    
                    // La cámara debe estar detrás de la nave (en dirección opuesta a forward)
                    let target_camera_pos = Vector3::new(
                        target_pos.x - camera_forward.x * ship_offset_forward - camera_up_dir.x * ship_offset_down,
                        target_pos.y - camera_forward.y * ship_offset_forward - camera_up_dir.y * ship_offset_down,
                        target_pos.z - camera_forward.z * ship_offset_forward - camera_up_dir.z * ship_offset_down,
                    );
                    
                    warp_system.start_warp(
                        elapsed_time,
                        ship.position,
                        target_pos,
                        camera.eye,
                        target_camera_pos,
                    );
                    break;
                }
            }
        }

        // Actualizar sistema de warping sobre la nave y la cámara
        let _warp_completed = warp_system.update(elapsed_time);
        
        if warp_system.is_warping {
            // Durante el warp, mover tanto la nave como la cámara
            ship.position = warp_system.get_current_ship_position();
            camera.eye = warp_system.get_current_camera_position();
            
            // Calcular dirección forward de la cámara basada en yaw y pitch
            let cos_yaw = camera.yaw.cos();
            let sin_yaw = camera.yaw.sin();
            let cos_pitch = camera.pitch.cos();
            let sin_pitch = camera.pitch.sin();
            
            let _camera_forward = Vector3::new(
                cos_yaw * cos_pitch,
                sin_pitch,
                sin_yaw * cos_pitch,
            );
            
            // Actualizar target de la cámara para que mire hacia la nave
            camera.target = Vector3::new(
                ship.position.x,
                ship.position.y,
                ship.position.z,
            );
            
            // La nave rota exactamente igual que la cámara (solo si no usa rotación directa)
            if !ship.use_direct_rotation {
                ship.rotation.y = camera.yaw;
                ship.rotation.x = camera.pitch;
                ship.rotation.z = 0.0;
            }
        } else {
            // Cuando no hay warp, comportamiento normal: la nave sigue a la cámara
            // La nave está completamente ligada al movimiento de la cámara
            // Se mueve y rota junto con la cámara, no está relacionada con los planetas
            // Calcular dirección forward de la cámara basada en yaw y pitch
            let cos_yaw = camera.yaw.cos();
            let sin_yaw = camera.yaw.sin();
            let cos_pitch = camera.pitch.cos();
            let sin_pitch = camera.pitch.sin();
            
            // Calcular dirección forward de la cámara (hacia donde mira)
            let camera_forward = Vector3::new(
                cos_yaw * cos_pitch,
                sin_pitch,
                sin_yaw * cos_pitch,
            );
            
            // Calcular dirección right de la cámara (perpendicular a forward)
            let camera_right = Vector3::new(
                -sin_yaw,
                0.0,
                cos_yaw,
            );
            
            // Calcular dirección up de la cámara
            let camera_up_dir = Vector3::new(
                -cos_yaw * sin_pitch,
                cos_pitch,
                -sin_yaw * sin_pitch,
            );
            
            // Cámara libre: La nave siempre está fija en la perspectiva de la cámara
            // La nave está "pegada" a la cámara en una posición relativa fija
            // Desde la perspectiva de la cámara, la nave siempre está en el mismo lugar en la pantalla
            let ship_offset_forward = 20.0; // Distancia fija delante de la cámara (zoom fijo)
            let ship_offset_down = -2.0; // Ligeramente abajo
            let ship_offset_right = 0.0; // Centrada horizontalmente
            
            // Posicionar la nave en una posición relativa fija respecto a la cámara
            // Esta posición es constante desde la perspectiva de la cámara
            // Cuando la cámara se mueve o rota, la nave se mueve y rota con ella instantáneamente
            ship.position = Vector3::new(
                camera.eye.x + camera_forward.x * ship_offset_forward + camera_right.x * ship_offset_right + camera_up_dir.x * ship_offset_down,
                camera.eye.y + camera_forward.y * ship_offset_forward + camera_right.y * ship_offset_right + camera_up_dir.y * ship_offset_down,
                camera.eye.z + camera_forward.z * ship_offset_forward + camera_right.z * ship_offset_right + camera_up_dir.z * ship_offset_down,
            );
            
            // La nave rota exactamente igual que la cámara (solo si no usa rotación directa)
            // Cuando rotas la cámara, la nave rota igual, manteniéndose en la misma posición relativa
            if !ship.use_direct_rotation {
                ship.rotation.y = camera.yaw; // Yaw de la cámara - rota con A/D
                ship.rotation.x = camera.pitch; // Pitch de la cámara - rota con W/S
                ship.rotation.z = 0.0; // Sin roll por ahora
            }
            
            // Vista tercera persona: La cámara siempre mira hacia la nave
            // Esto asegura que la nave esté siempre visible y en la misma posición en la pantalla
            camera.target = Vector3::new(
                ship.position.x,
                ship.position.y,
                ship.position.z,
            );
        }
        
        // Actualizar parámetros de la cámara
        camera.up = Vector3::new(0.0, 1.0, 0.0);
        
        // La distancia es fija (zoom fijo), no necesita recalcularse
        // camera.distance se mantiene en 20.0 (definido en process_input)

        // Update orbital positions and rotations
        for planet in &mut planets {
            planet.orbital_angle += planet.orbital_speed * delta_time;
            if planet.orbital_angle >= 2.0 * PI {
                planet.orbital_angle -= 2.0 * PI;
            }
        }

        framebuffer.clear();

        // Dibujar estrellas en el skybox (fondo negro con puntos blancos)
        // Usar una profundidad muy lejana para que las estrellas estén detrás de todo
        for &(star_x, star_y, brightness) in &stars {
            let star_color = Vector3::new(brightness, brightness, brightness);
            framebuffer.point(star_x, star_y, star_color, 999.0);
        }

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

        // Renderizar la nave
        let ship_translation = ship.position;
        // Aplicar rotación del modelo: la nave rota exactamente igual que la cámara
        // La rotación ya está sincronizada con la cámara (ship.rotation = camera.yaw/pitch)
        // El modelo puede necesitar una orientación inicial, pero la rotación debe seguir a la cámara
        let ship_rotation = Vector3::new(
            ship.rotation.x,  // Pitch (sincronizado con camera.pitch) - rota con W/S
            ship.rotation.y,   // Yaw (sincronizado con camera.yaw) - rota con A/D
            ship.rotation.z,   // Roll
        );
        let ship_model_matrix = create_model_matrix(ship_translation, ship.scale, ship_rotation);
        
        let ship_uniforms = Uniforms {
            model_matrix: ship_model_matrix,
            view_matrix,
            projection_matrix,
            viewport_matrix,
            time: elapsed_time,
        };

        // Renderizar la nave - siempre visible ya que la cámara la sigue
        // La nave siempre está en la escena
        // Usar shader gris mejorado para la nave con mejor visibilidad
        if !ship_vertex_array.is_empty() {
            render(&mut framebuffer, &ship_uniforms, &ship_vertex_array, &light, PlanetType::Ship);
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
