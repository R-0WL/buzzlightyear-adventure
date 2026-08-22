# Star Command Adventure Labyrinth — Ray Caster en Rust

Proyecto jugable de ray casting realizado en **Rust** con Macroquad. Incluye tres
laberintos originales de 24 × 18 celdas, paredes con materiales diferentes,
colisiones, cámara en primera persona, minimapa, audio, animación y condición de
victoria.

## Requisitos

- Rust estable, instalado desde <https://rustup.rs/>
- Windows, macOS o Linux
- En Linux pueden ser necesarios los paquetes de desarrollo de ALSA y X11:

```bash
sudo apt install pkg-config libx11-dev libxi-dev libgl1-mesa-dev libasound2-dev
```

## Ejecutar

```bash
git clone https://github.com/R-0WL/buzzlightyear-adventure
cd buzzlightyear-adventure
cargo run --release
```

 También se puede comprobar el proyecto con:

```bash
cargo test
cargo fmt --check
cargo clippy -- -D warnings
```

## Controles

| Tecla | Acción |
|---|---|
| `W` / `S` | Avanzar / retroceder |
| `A` / `D` | Desplazarse lateralmente |
| Mouse horizontal | Rotar la cámara |
| `Q` / `E` o flechas izquierda/derecha | Rotación alternativa |
| `M` | Activar o silenciar la música |
| `Esc` | Volver al menú |
| `R` | Reiniciar después de ganar |
| `Enter` | Confirmar nivel |

## Objetivo del juego

1. Busca el **núcleo azul animado**.
2. Recógelo al acercarte.
3. Dirígete al **portal violeta** para escapar.

El minimapa de la esquina superior derecha muestra paredes, jugador, núcleo y
portal. El triángulo amarillo indica la posición y dirección del jugador.

## Relación con la rúbrica

| Requisito | Implementación |
|---|---|
| Nivel entero y jugable | Tres mapas originales de 24 × 18 |
| No atravesar paredes | Colisión circular, resolución independiente por eje y bordes seguros |
| Paredes diferentes | Cuatro materiales procedurales: piedra, ladrillo, musgo y tecnología |
| FPS | Contador visible; renderizado optimizado a una columna por píxel |
| Cámara | Movimiento WASD, giro por teclado y rotación horizontal con mouse |
| Minimapa | Superpuesto en la esquina superior derecha |
| Música | Tema chiptune original, sintetizado en memoria y reproducido en bucle |
| Efectos | Golpe contra pared, recolección y victoria |
| Sprite animado | Núcleo flotante con cuatro cuadros, brillo, alas y rotación |
| Bienvenida | Menú visual con selección de tres niveles |
| Éxito | Pantalla final con tiempo y opciones de reinicio |
| Estética | Tema neón, cielo y piso, texturas procedurales, HUD y portal animado |

## Arquitectura

```text
src/
├── main.rs       estados, interfaz, minimapa y ciclo principal
├── level.rs      mapas, consultas seguras y validación de rutas
├── player.rs     movimiento, cámara y colisiones
├── raycaster.rs  algoritmo DDA, paredes, profundidad y sprites
└── audio.rs      música y efectos WAV generados en memoria
```

El algoritmo lanza un rayo por cada columna de la pantalla. DDA avanza por la
cuadrícula hasta encontrar una pared; la distancia determina la altura de esa
columna. Un búfer de profundidad evita dibujar el núcleo o el portal frente a
una pared que realmente los oculta.
