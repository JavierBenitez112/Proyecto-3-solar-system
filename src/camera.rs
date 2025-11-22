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
            rotation_speed: 0.02,  // Velocidad de rotación reducida
            zoom_speed: 0.2,        // Velocidad de zoom reducida
            pan_speed: 0.15,       // Velocidad de movimiento con flechas (aumentada)
            tracking_planet: None, // Inicialmente no sigue ningún planeta
            ecliptic_height,
        }
    }

    /// Update camera eye position based on yaw, pitch, and distance
    /// Restringe el movimiento al plano eclíptico (Y constante)
    pub fn update_eye_position(&mut self) {
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

    /// Process keyboard input to control the camera libre (FPS-style)
    /// Cámara libre que se desplaza por el skybox con zoom fijo
    pub fn process_input(&mut self, window: &RaylibHandle) {
        // Calcular direcciones de la cámara basadas en yaw y pitch
        let cos_yaw = self.yaw.cos();
        let sin_yaw = self.yaw.sin();
        let cos_pitch = self.pitch.cos();
        let sin_pitch = self.pitch.sin();
        
        // Dirección forward de la cámara
        let forward = Vector3::new(
            cos_yaw * cos_pitch,
            sin_pitch,
            sin_yaw * cos_pitch,
        );
        
        // Dirección right de la cámara
        let right = Vector3::new(
            -sin_yaw,
            0.0,
            cos_yaw,
        );
        
        // Dirección up de la cámara (no se usa actualmente, pero se mantiene para futuras extensiones)
        let _up = Vector3::new(
            -cos_yaw * sin_pitch,
            cos_pitch,
            -sin_yaw * sin_pitch,
        );

        // Rotation controls (yaw) - A/D
        if window.is_key_down(KeyboardKey::KEY_A) {
            self.yaw += self.rotation_speed;
        }
        if window.is_key_down(KeyboardKey::KEY_D) {
            self.yaw -= self.rotation_speed;
        }

        // Rotation controls (pitch) - W/S
        if window.is_key_down(KeyboardKey::KEY_W) {
            self.pitch += self.rotation_speed;
            self.pitch = self.pitch.clamp(-PI / 2.0 + 0.1, PI / 2.0 - 0.1); // Limitar pitch
        }
        if window.is_key_down(KeyboardKey::KEY_S) {
            self.pitch -= self.rotation_speed;
            self.pitch = self.pitch.clamp(-PI / 2.0 + 0.1, PI / 2.0 - 0.1); // Limitar pitch
        }

        // Movimiento libre de la cámara (desplazamiento por el skybox)
        // Q/E para movimiento lateral
        if window.is_key_down(KeyboardKey::KEY_Q) {
            self.eye.x -= right.x * self.pan_speed;
            self.eye.z -= right.z * self.pan_speed;
        }
        if window.is_key_down(KeyboardKey::KEY_E) {
            self.eye.x += right.x * self.pan_speed;
            self.eye.z += right.z * self.pan_speed;
        }

        // Left/Right arrow keys para movimiento lateral
        if window.is_key_down(KeyboardKey::KEY_LEFT) {
            self.eye.x -= right.x * self.pan_speed;
            self.eye.z -= right.z * self.pan_speed;
        }
        if window.is_key_down(KeyboardKey::KEY_RIGHT) {
            self.eye.x += right.x * self.pan_speed;
            self.eye.z += right.z * self.pan_speed;
        }

        // Up/Down arrow keys para movimiento forward/backward
        if window.is_key_down(KeyboardKey::KEY_UP) {
            self.eye.x += forward.x * self.pan_speed;
            self.eye.y += forward.y * self.pan_speed;
            self.eye.z += forward.z * self.pan_speed;
        }
        if window.is_key_down(KeyboardKey::KEY_DOWN) {
            self.eye.x -= forward.x * self.pan_speed;
            self.eye.y -= forward.y * self.pan_speed;
            self.eye.z -= forward.z * self.pan_speed;
        }

        // R/F para movimiento vertical
        if window.is_key_down(KeyboardKey::KEY_R) {
            self.eye.y += self.pan_speed;
        }
        if window.is_key_down(KeyboardKey::KEY_F) {
            self.eye.y -= self.pan_speed;
        }

        // Zoom fijo - no se permite cambiar la distancia
        // La distancia se mantiene constante
        self.distance = 20.0; // Distancia fija para vista tercera persona
        
        // El target se actualiza en main.rs después de posicionar la nave
        // No actualizamos el target aquí para evitar conflictos
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
