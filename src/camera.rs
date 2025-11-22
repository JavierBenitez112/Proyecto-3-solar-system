#![allow(dead_code)]

use raylib::prelude::*;
use crate::matrix::create_view_matrix;
use std::f32::consts::PI;

pub struct Camera {
    // Camera position/orientation
    pub eye: Vector3,        // Camera position
    pub target: Vector3,     // Point the camera is looking at
    pub up: Vector3,         // Up vector

    // Orbit camera parameters
    pub yaw: f32,            // Rotation around Y axis (left/right)
    pub pitch: f32,          // Rotation around X axis (up/down)
    pub distance: f32,       // Distance from target

    // Movement speed
    pub rotation_speed: f32,
    pub zoom_speed: f32,
    pub pan_speed: f32,

    // Planet tracking
    pub tracking_planet: Option<usize>, // Índice del planeta que se está siguiendo (None = modo libre)
    pub ecliptic_height: f32, // Altura fija sobre el plano eclíptico
}

impl Camera {
    pub fn new(eye: Vector3, target: Vector3, up: Vector3) -> Self {
        // Calculate initial yaw and pitch from eye and target
        let direction = Vector3::new(
            eye.x - target.x,
            eye.y - target.y,
            eye.z - target.z,
        );

        let distance = (direction.x * direction.x + direction.y * direction.y + direction.z * direction.z).sqrt();
        let pitch = (direction.y / distance).asin();
        let yaw = direction.z.atan2(direction.x);

        // Calcular altura inicial sobre el plano eclíptico (Y=0)
        let ecliptic_height = eye.y;

        Camera {
            eye,
            target,
            up,
            yaw,
            pitch,
            distance,
            rotation_speed: 0.05,
            zoom_speed: 0.5,
            pan_speed: 0.1,
            tracking_planet: None, // Inicialmente no sigue ningún planeta
            ecliptic_height,
        }
    }

    /// Update camera eye position based on yaw, pitch, and distance
    /// Restringe el movimiento al plano eclíptico (Y constante)
    fn update_eye_position(&mut self) {
        // Restringir pitch para mantener la cámara en el plano eclíptico
        // Permitir solo un pequeño ángulo para ver el plano desde arriba
        self.pitch = self.pitch.clamp(-PI / 6.0, PI / 6.0); // Máximo 30 grados arriba/abajo

        // Calcular posición de la cámara en el plano eclíptico
        // La altura Y se mantiene constante (ecliptic_height)
        let horizontal_distance = self.distance * self.pitch.cos();
        self.eye.x = self.target.x + horizontal_distance * self.yaw.cos();
        self.eye.y = self.target.y + self.ecliptic_height; // Altura fija sobre el plano eclíptico
        self.eye.z = self.target.z + horizontal_distance * self.yaw.sin();
    }

    /// Configurar la cámara para seguir un planeta específico
    pub fn track_planet(&mut self, planet_index: Option<usize>) {
        self.tracking_planet = planet_index;
    }

    /// Actualizar el target para seguir el planeta que se está rastreando
    /// Si tracking_planet es None, sigue el sol (centro)
    pub fn update_planet_tracking(&mut self, planet_position: Vector3) {
        // Suavizar el seguimiento del planeta o sol
        let smoothing = 0.1;
        self.target.x += (planet_position.x - self.target.x) * smoothing;
        self.target.y = planet_position.y; // Mantener en el plano eclíptico (Y=0)
        self.target.z += (planet_position.z - self.target.z) * smoothing;
    }

    /// Get the view matrix for this camera
    pub fn get_view_matrix(&self) -> Matrix {
        create_view_matrix(self.eye, self.target, self.up)
    }

    /// Process keyboard input to control the camera
    pub fn process_input(&mut self, window: &RaylibHandle) {
        // Rotation controls (yaw)
        if window.is_key_down(KeyboardKey::KEY_A) {
            self.yaw += self.rotation_speed;
            self.update_eye_position();
        }
        if window.is_key_down(KeyboardKey::KEY_D) {
            self.yaw -= self.rotation_speed;
            self.update_eye_position();
        }

        // Rotation controls (pitch)
        if window.is_key_down(KeyboardKey::KEY_W) {
            self.pitch += self.rotation_speed;
            self.update_eye_position();
        }
        if window.is_key_down(KeyboardKey::KEY_S) {
            self.pitch -= self.rotation_speed;
            self.update_eye_position();
        }

        // Zoom controls (distance from target) - arrow keys
        if window.is_key_down(KeyboardKey::KEY_UP) {
            self.distance -= self.zoom_speed;
            if self.distance < 0.5 {
                self.distance = 0.5; // Prevent camera from going too close
            }
            self.update_eye_position();
        }
        if window.is_key_down(KeyboardKey::KEY_DOWN) {
            self.distance += self.zoom_speed;
            self.update_eye_position();
        }

        // Pan controls (move target/center point)
        // Calculate right and forward vectors for panning
        let forward = Vector3::new(
            self.target.x - self.eye.x,
            0.0, // Keep on horizontal plane
            self.target.z - self.eye.z,
        );
        let forward_len = (forward.x * forward.x + forward.z * forward.z).sqrt();
        let forward_normalized = if forward_len > 0.0 {
            Vector3::new(forward.x / forward_len, 0.0, forward.z / forward_len)
        } else {
            Vector3::new(0.0, 0.0, 1.0)
        };

        let right = Vector3::new(
            forward_normalized.z,
            0.0,
            -forward_normalized.x,
        );

        // Q/E keys for horizontal panning
        if window.is_key_down(KeyboardKey::KEY_Q) {
            self.target.x -= right.x * self.pan_speed;
            self.target.z -= right.z * self.pan_speed;
            self.update_eye_position();
        }
        if window.is_key_down(KeyboardKey::KEY_E) {
            self.target.x += right.x * self.pan_speed;
            self.target.z += right.z * self.pan_speed;
            self.update_eye_position();
        }

        // Left/Right arrow keys for horizontal panning
        if window.is_key_down(KeyboardKey::KEY_LEFT) {
            self.target.x -= right.x * self.pan_speed;
            self.target.z -= right.z * self.pan_speed;
            self.update_eye_position();
        }
        if window.is_key_down(KeyboardKey::KEY_RIGHT) {
            self.target.x += right.x * self.pan_speed;
            self.target.z += right.z * self.pan_speed;
            self.update_eye_position();
        }

        // Vertical panning - Restringido para mantener el plano eclíptico
        // Solo permitir ajuste fino de altura sobre el plano
        if window.is_key_down(KeyboardKey::KEY_R) {
            self.ecliptic_height += self.pan_speed * 0.5; // Movimiento más lento
            self.ecliptic_height = self.ecliptic_height.clamp(5.0, 50.0); // Limitar altura
            self.update_eye_position();
        }
        if window.is_key_down(KeyboardKey::KEY_F) {
            self.ecliptic_height -= self.pan_speed * 0.5;
            self.ecliptic_height = self.ecliptic_height.clamp(5.0, 50.0);
            self.update_eye_position();
        }

        // Restringir el target al plano eclíptico (Y=0)
        self.target.y = 0.0;
    }

    /// Obtener el índice del planeta que se está siguiendo
    pub fn get_tracking_planet(&self) -> Option<usize> {
        self.tracking_planet
    }

    /// Teletransportar la cámara a una nueva posición y target
    pub fn warp_to(&mut self, new_position: Vector3, new_target: Vector3) {
        // Calcular nueva distancia y ángulos
        let direction = Vector3::new(
            new_position.x - new_target.x,
            new_position.y - new_target.y,
            new_position.z - new_target.z,
        );
        let distance = (direction.x * direction.x + direction.y * direction.y + direction.z * direction.z).sqrt();
        let pitch = (direction.y / distance.max(0.0001)).asin();
        let yaw = direction.z.atan2(direction.x);

        // Actualizar posición y parámetros
        self.eye = new_position;
        self.target = new_target;
        self.distance = distance;
        self.pitch = pitch;
        self.yaw = yaw;
        self.ecliptic_height = new_position.y;
        
        // Actualizar posición del ojo basada en los nuevos parámetros
        self.update_eye_position();
    }
}
