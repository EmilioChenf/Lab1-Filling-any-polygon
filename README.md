# Lab 1: Filling any polygon

Laboratorio en Rust que rasteriza polígonos convexos y cóncavos sin dependencias externas. La salida oficial es `out.bmp`, una imagen RGB de 24 bits y 800 × 500 píxeles.

## Algoritmos

El relleno utiliza Scanline con regla par-impar. En cada centro de fila se calculan y ordenan las intersecciones con las aristas no horizontales. Los vértices se manejan con una convención semiabierta: el extremo inferior se incluye y el superior se excluye. Así se evita contar dos veces los vértices compartidos.

Los contornos se rasterizan con Bresenham general. El algoritmo funciona en los ocho octantes, incluye ambos extremos y conecta el último vértice con el primero.

Una máscara contiene exactamente los píxeles recorridos por Bresenham. Una clasificación por inundación desde los límites identifica el exterior; las celdas encerradas forman el interior rasterizado. El relleno nunca sustituye píxeles de contorno.

`Poligon-5` es una exclusión real dentro de `Poligon-4`. Antes de escribir verde se comprueba que el píxel pertenezca al interior de `Poligon-4`, no pertenezca al interior ni al contorno de `Poligon-5` y no sea parte del contorno exterior. El agujero nunca se pinta ni se restaura con el color del fondo: conserva el valor con el que se inicializó el framebuffer. El reporte instrumental `FillReport` permite comprobar que el número de escrituras de relleno dentro del agujero es exactamente cero.

## Colores

- `Poligon-1`: amarillo con borde blanco.
- `Poligon-2`: azul con borde blanco.
- `Poligon-3`: rojo con borde blanco.
- `Poligon-4`: verde con borde blanco.
- `Poligon-5`: agujero sin pintar con borde blanco.
- Fondo: verde azulado oscuro, RGB `(8, 42, 45)`.

## Compilar, probar y ejecutar

Se requiere una instalación estable de Rust y Cargo.

```powershell
cargo fmt --check
cargo check
cargo test
cargo run
```

`cargo run` genera `out.bmp` en la raíz del proyecto.

## Ramas

- `Poligon-1`: escena aislada del polígono 1.
- `Poligon-2`: escena aislada del polígono 2.
- `Poligon-3`: escena aislada del polígono 3.
- `Poligon-4`: escena aislada del polígono 4 con el polígono 5 como agujero real.
- `main`: escena final con los cuatro polígonos y el agujero.

El commit inicial fue proporcionado como base. Todos los cambios posteriores incorporados a main corresponden exclusivamente a merges de las cuatro ramas requeridas.

## Estructura

```text
src/
  main.rs         Coordenadas, colores, composición y pruebas de la escena
  framebuffer.rs  Framebuffer RGB y escritura segura de píxeles
  line.rs         Rasterización de líneas con Bresenham
  polygon.rs      Scanline, regla par-impar y máscaras de contorno
  bmp.rs          Escritura de BMP de 24 bits
```

`target/`, `build/`, ejecutables, objetos y cachés están excluidos mediante `.gitignore`. `out.bmp` sí forma parte de la entrega.
