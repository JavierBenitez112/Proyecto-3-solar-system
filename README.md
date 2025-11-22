# Sistema Solar 3D

Simulador de sistema solar con nave espacial explorable en Rust usando Raylib.

## Video
https://youtu.be/R_lt5QnPfRA


image.png
image.png


## Requisitos

- Rust (última versión estable)
- Cargo

## Ejecutar

```bash
cargo run
```

## Controles

### Cámara/Nave
- **W/S** - Rotar arriba/abajo (pitch)
- **A/D** - Rotar izquierda/derecha (yaw)
- **↑/↓** - Avanzar/retroceder
- **←/→** - Movimiento lateral
- **Q/E** - Movimiento lateral alternativo
- **R/F** - Subir/bajar

### Teletransporte
- **F1** - Vista general del sistema
- **F2** - Cerca del Sol
- **F3-F7** - Teletransportarse a cada planeta

## Características

- Sistema solar con 5 planetas orbitando
- Nave espacial 3D controlable
- Cámara en tercera persona estilo Star Fox 64
- Skybox con estrellas
- Shaders procedurales para planetas
- Sistema de teletransporte (warp)

## Modelo 3D

La nave usa el modelo `assets/models/Untitled.obj`. El modelo se puede rotar programáticamente usando los métodos de la estructura `Ship`.

