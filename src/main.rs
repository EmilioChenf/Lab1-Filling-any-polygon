mod bmp;
mod framebuffer;
mod line;
mod polygon;

use std::io;

use bmp::write_bmp;
use framebuffer::{Color, Framebuffer};
use polygon::{Point, build_outline_mask, draw_polygon_outline, fill_polygon_scanline};

const WIDTH: usize = 800;
const HEIGHT: usize = 500;
const COLOR_MENTA_PAPATIN: Color = Color::rgb(8, 42, 45);
const WHITE: Color = Color::rgb(255, 255, 255);
const YELLOW: Color = Color::rgb(255, 255, 0);
const BLUE: Color = Color::rgb(0, 90, 255);
const RED: Color = Color::rgb(255, 0, 0);
const GREEN: Color = Color::rgb(0, 180, 70);

const POLYGON_1: [Point; 10] = [
    Point::new(165, 380),
    Point::new(185, 360),
    Point::new(180, 330),
    Point::new(207, 345),
    Point::new(233, 330),
    Point::new(230, 360),
    Point::new(250, 380),
    Point::new(220, 385),
    Point::new(205, 410),
    Point::new(193, 383),
];

const POLYGON_2: [Point; 4] = [
    Point::new(321, 335),
    Point::new(288, 286),
    Point::new(339, 251),
    Point::new(374, 302),
];

const POLYGON_3: [Point; 3] = [
    Point::new(377, 249),
    Point::new(411, 197),
    Point::new(436, 249),
];

const POLYGON_4: [Point; 18] = [
    Point::new(413, 177),
    Point::new(448, 159),
    Point::new(502, 88),
    Point::new(553, 53),
    Point::new(535, 36),
    Point::new(676, 37),
    Point::new(660, 52),
    Point::new(750, 145),
    Point::new(761, 179),
    Point::new(672, 192),
    Point::new(659, 214),
    Point::new(615, 214),
    Point::new(632, 230),
    Point::new(580, 230),
    Point::new(597, 215),
    Point::new(552, 214),
    Point::new(517, 144),
    Point::new(466, 180),
];

const POLYGON_5_HOLE: [Point; 4] = [
    Point::new(682, 175),
    Point::new(708, 120),
    Point::new(735, 148),
    Point::new(739, 170),
];

fn render_scene() -> Framebuffer {
    let mut framebuffer = Framebuffer::new(WIDTH, HEIGHT, COLOR_MENTA_PAPATIN);
    let mask_1 = build_outline_mask(WIDTH, HEIGHT, &[&POLYGON_1]);
    let mask_2 = build_outline_mask(WIDTH, HEIGHT, &[&POLYGON_2]);
    let mask_3 = build_outline_mask(WIDTH, HEIGHT, &[&POLYGON_3]);
    let mask_4 = build_outline_mask(WIDTH, HEIGHT, &[&POLYGON_4]);
    let mask_5 = build_outline_mask(WIDTH, HEIGHT, &[&POLYGON_5_HOLE]);

    fill_polygon_scanline(&mut framebuffer, &POLYGON_1, &[], YELLOW, &mask_1);
    fill_polygon_scanline(&mut framebuffer, &POLYGON_2, &[], BLUE, &mask_2);
    fill_polygon_scanline(&mut framebuffer, &POLYGON_3, &[], RED, &mask_3);
    let polygon_4_fill_report = fill_polygon_scanline(
        &mut framebuffer,
        &POLYGON_4,
        &[(&POLYGON_5_HOLE, &mask_5)],
        GREEN,
        &mask_4,
    );
    debug_assert_eq!(polygon_4_fill_report.fill_writes_inside_holes, 0);

    // Los contornos se trazan al final para permanecer nitidos sobre el relleno.
    draw_polygon_outline(&mut framebuffer, &POLYGON_1, WHITE);
    draw_polygon_outline(&mut framebuffer, &POLYGON_2, WHITE);
    draw_polygon_outline(&mut framebuffer, &POLYGON_3, WHITE);
    draw_polygon_outline(&mut framebuffer, &POLYGON_4, WHITE);
    draw_polygon_outline(&mut framebuffer, &POLYGON_5_HOLE, WHITE);
    framebuffer
}

fn render_single_polygon(vertices: &[Point], color: Color) -> Framebuffer {
    let mut framebuffer = Framebuffer::new(WIDTH, HEIGHT, COLOR_MENTA_PAPATIN);
    let mask = build_outline_mask(WIDTH, HEIGHT, &[vertices]);
    fill_polygon_scanline(&mut framebuffer, vertices, &[], color, &mask);
    draw_polygon_outline(&mut framebuffer, vertices, WHITE);
    framebuffer
}

fn render_polygon_1_scene() -> Framebuffer {
    render_single_polygon(&POLYGON_1, YELLOW)
}

fn render_polygon_2_scene() -> Framebuffer {
    render_single_polygon(&POLYGON_2, BLUE)
}

fn render_polygon_3_scene() -> Framebuffer {
    render_single_polygon(&POLYGON_3, RED)
}

fn render_polygon_4_scene() -> Framebuffer {
    let mut framebuffer = Framebuffer::new(WIDTH, HEIGHT, COLOR_MENTA_PAPATIN);
    let outer_mask = build_outline_mask(WIDTH, HEIGHT, &[&POLYGON_4]);
    let hole_mask = build_outline_mask(WIDTH, HEIGHT, &[&POLYGON_5_HOLE]);
    let report = fill_polygon_scanline(
        &mut framebuffer,
        &POLYGON_4,
        &[(&POLYGON_5_HOLE, &hole_mask)],
        GREEN,
        &outer_mask,
    );
    debug_assert_eq!(report.fill_writes_inside_holes, 0);
    draw_polygon_outline(&mut framebuffer, &POLYGON_4, WHITE);
    draw_polygon_outline(&mut framebuffer, &POLYGON_5_HOLE, WHITE);
    framebuffer
}

const SCENE_RENDERERS: [fn() -> Framebuffer; 5] = [
    render_polygon_1_scene,
    render_polygon_2_scene,
    render_polygon_3_scene,
    render_polygon_4_scene,
    render_scene,
];

fn main() -> io::Result<()> {
    let framebuffer = SCENE_RENDERERS[0]();
    write_bmp("out.bmp", &framebuffer)?;
    println!("Imagen generada: out.bmp ({WIDTH}x{HEIGHT}) - Poligon-1");
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;

    use super::*;

    #[test]
    fn agujero_recibe_cero_escrituras_de_relleno() {
        let mut framebuffer = Framebuffer::new(WIDTH, HEIGHT, COLOR_MENTA_PAPATIN);
        let outer_mask = build_outline_mask(WIDTH, HEIGHT, &[&POLYGON_4]);
        let hole_mask = build_outline_mask(WIDTH, HEIGHT, &[&POLYGON_5_HOLE]);
        let report = fill_polygon_scanline(
            &mut framebuffer,
            &POLYGON_4,
            &[(&POLYGON_5_HOLE, &hole_mask)],
            GREEN,
            &outer_mask,
        );
        assert!(report.total_fill_writes > 0);
        assert_eq!(report.fill_writes_inside_holes, 0);
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                if hole_mask.is_interior(x, y) {
                    assert_eq!(framebuffer.pixel(x, y), COLOR_MENTA_PAPATIN);
                }
            }
        }
    }

    #[test]
    fn relleno_completo_coincide_con_el_interior_rasterizado() {
        let framebuffer = render_scene();
        let masks = [
            build_outline_mask(WIDTH, HEIGHT, &[&POLYGON_1]),
            build_outline_mask(WIDTH, HEIGHT, &[&POLYGON_2]),
            build_outline_mask(WIDTH, HEIGHT, &[&POLYGON_3]),
            build_outline_mask(WIDTH, HEIGHT, &[&POLYGON_4]),
        ];
        let hole_mask = build_outline_mask(WIDTH, HEIGHT, &[&POLYGON_5_HOLE]);
        let mut counts = [0_usize; 4];
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                let color = framebuffer.pixel(x, y);
                let mask_index = if color == YELLOW {
                    counts[0] += 1;
                    Some(0)
                } else if color == BLUE {
                    counts[1] += 1;
                    Some(1)
                } else if color == RED {
                    counts[2] += 1;
                    Some(2)
                } else if color == GREEN {
                    counts[3] += 1;
                    assert!(!hole_mask.is_interior(x, y));
                    assert!(!hole_mask.is_border(x, y));
                    Some(3)
                } else {
                    None
                };
                if let Some(mask_index) = mask_index {
                    assert!(!masks[mask_index].is_border(x, y));
                    assert!(
                        masks[mask_index].is_interior(x, y),
                        "relleno fuera de la barrera Bresenham en ({x}, {y})"
                    );
                }

                if masks[0].is_interior(x, y) {
                    assert_eq!(color, YELLOW, "hueco amarillo en ({x}, {y})");
                }
                if masks[1].is_interior(x, y) {
                    assert_eq!(color, BLUE, "hueco azul en ({x}, {y})");
                }
                if masks[2].is_interior(x, y) {
                    assert_eq!(color, RED, "hueco rojo en ({x}, {y})");
                }
                if masks[3].is_interior(x, y)
                    && !hole_mask.is_interior(x, y)
                    && !hole_mask.is_border(x, y)
                {
                    assert_eq!(color, GREEN, "hueco verde en ({x}, {y})");
                }
                if hole_mask.is_interior(x, y) {
                    assert_eq!(color, COLOR_MENTA_PAPATIN, "agujero pintado en ({x}, {y})");
                }
            }
        }
        assert!(counts.into_iter().all(|count| count > 0));
    }

    #[test]
    fn todos_los_pixeles_bresenham_terminan_blancos() {
        let framebuffer = render_scene();
        let contours: [&[Point]; 5] = [
            &POLYGON_1,
            &POLYGON_2,
            &POLYGON_3,
            &POLYGON_4,
            &POLYGON_5_HOLE,
        ];
        let mask = build_outline_mask(WIDTH, HEIGHT, &contours);
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                if mask.is_border(x, y) {
                    assert_eq!(framebuffer.pixel(x, y), WHITE, "borde no blanco ({x}, {y})");
                }
            }
        }
    }

    #[test]
    fn interiores_exteriores_y_agujero_tienen_el_color_correcto() {
        let framebuffer = render_scene();
        assert_eq!(framebuffer.pixel(205, 370), YELLOW);
        assert_eq!(framebuffer.pixel(330, 290), BLUE);
        assert_eq!(framebuffer.pixel(411, 225), RED);
        assert_eq!(framebuffer.pixel(600, 100), GREEN);

        // Pixeles inmediatamente exteriores a diagonales representativas.
        assert_eq!(framebuffer.pixel(382, 240), COLOR_MENTA_PAPATIN);
        assert_eq!(framebuffer.pixel(433, 240), COLOR_MENTA_PAPATIN);
        assert_eq!(framebuffer.pixel(296, 300), COLOR_MENTA_PAPATIN);

        for (x, y) in [(710, 145), (715, 155), (725, 160)] {
            assert_eq!(framebuffer.pixel(x, y), COLOR_MENTA_PAPATIN);
        }
    }

    fn renderizar_aislado(vertices: &[Point], color: Color) -> Framebuffer {
        let mut framebuffer = Framebuffer::new(WIDTH, HEIGHT, COLOR_MENTA_PAPATIN);
        let mask = build_outline_mask(WIDTH, HEIGHT, &[vertices]);
        fill_polygon_scanline(&mut framebuffer, vertices, &[], color, &mask);
        draw_polygon_outline(&mut framebuffer, vertices, WHITE);
        framebuffer
    }

    fn fondo_no_alcanzable(framebuffer: &Framebuffer) -> Vec<bool> {
        let mut reachable = vec![false; WIDTH * HEIGHT];
        let mut queue = VecDeque::new();
        let enqueue = |x: usize,
                       y: usize,
                       reachable: &mut Vec<bool>,
                       queue: &mut VecDeque<(usize, usize)>| {
            let index = y * WIDTH + x;
            if !reachable[index] && framebuffer.pixel(x, y) == COLOR_MENTA_PAPATIN {
                reachable[index] = true;
                queue.push_back((x, y));
            }
        };
        for x in 0..WIDTH {
            enqueue(x, 0, &mut reachable, &mut queue);
            enqueue(x, HEIGHT - 1, &mut reachable, &mut queue);
        }
        for y in 0..HEIGHT {
            enqueue(0, y, &mut reachable, &mut queue);
            enqueue(WIDTH - 1, y, &mut reachable, &mut queue);
        }
        while let Some((x, y)) = queue.pop_front() {
            if x > 0 {
                enqueue(x - 1, y, &mut reachable, &mut queue);
            }
            if x + 1 < WIDTH {
                enqueue(x + 1, y, &mut reachable, &mut queue);
            }
            if y > 0 {
                enqueue(x, y - 1, &mut reachable, &mut queue);
            }
            if y + 1 < HEIGHT {
                enqueue(x, y + 1, &mut reachable, &mut queue);
            }
        }
        reachable
            .into_iter()
            .enumerate()
            .map(|(index, reached)| {
                !reached && framebuffer.pixel(index % WIDTH, index / WIDTH) == COLOR_MENTA_PAPATIN
            })
            .collect()
    }

    #[test]
    fn no_hay_componentes_de_fondo_atrapadas_salvo_el_agujero() {
        for (vertices, color) in [
            (&POLYGON_1[..], YELLOW),
            (&POLYGON_2[..], BLUE),
            (&POLYGON_3[..], RED),
        ] {
            let framebuffer = renderizar_aislado(vertices, color);
            assert_eq!(
                fondo_no_alcanzable(&framebuffer)
                    .iter()
                    .filter(|v| **v)
                    .count(),
                0
            );
        }

        let framebuffer = render_scene();
        let trapped = fondo_no_alcanzable(&framebuffer);
        let hole_mask = build_outline_mask(WIDTH, HEIGHT, &[&POLYGON_5_HOLE]);
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                assert_eq!(
                    trapped[y * WIDTH + x],
                    hole_mask.is_interior(x, y),
                    "componente de fondo inesperada en ({x}, {y})"
                );
            }
        }
    }

    fn vecinos_8(x: usize, y: usize) -> impl Iterator<Item = (usize, usize)> {
        let min_x = x.saturating_sub(1);
        let min_y = y.saturating_sub(1);
        let max_x = (x + 1).min(WIDTH - 1);
        let max_y = (y + 1).min(HEIGHT - 1);
        (min_y..=max_y)
            .flat_map(move |neighbor_y| {
                (min_x..=max_x).map(move |neighbor_x| (neighbor_x, neighbor_y))
            })
            .filter(move |&(neighbor_x, neighbor_y)| neighbor_x != x || neighbor_y != y)
    }

    #[test]
    fn el_relleno_toca_el_lado_interior_de_cada_contorno() {
        for (vertices, color) in [
            (&POLYGON_1[..], YELLOW),
            (&POLYGON_2[..], BLUE),
            (&POLYGON_3[..], RED),
            (&POLYGON_4[..], GREEN),
        ] {
            let framebuffer = renderizar_aislado(vertices, color);
            let mask = build_outline_mask(WIDTH, HEIGHT, &[vertices]);
            for y in 0..HEIGHT {
                for x in 0..WIDTH {
                    if mask.is_border(x, y)
                        && vecinos_8(x, y).any(|(nx, ny)| mask.is_interior(nx, ny))
                    {
                        assert!(
                            vecinos_8(x, y).any(|(nx, ny)| {
                                let pixel = framebuffer.pixel(nx, ny);
                                pixel == color || pixel == WHITE
                            }),
                            "fondo entre contorno y relleno en ({x}, {y})"
                        );
                    }
                }
            }
        }

        let framebuffer = render_scene();
        let hole_mask = build_outline_mask(WIDTH, HEIGHT, &[&POLYGON_5_HOLE]);
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                if hole_mask.is_border(x, y) {
                    assert!(
                        vecinos_8(x, y).any(|(nx, ny)| {
                            let pixel = framebuffer.pixel(nx, ny);
                            pixel == GREEN || pixel == WHITE
                        }),
                        "fondo entre verde y borde del agujero en ({x}, {y})"
                    );
                }
            }
        }
    }

    #[test]
    fn ningun_pixel_de_mascara_conserva_relleno() {
        let framebuffer = render_scene();
        let contours: [&[Point]; 5] = [
            &POLYGON_1,
            &POLYGON_2,
            &POLYGON_3,
            &POLYGON_4,
            &POLYGON_5_HOLE,
        ];
        let mask = build_outline_mask(WIDTH, HEIGHT, &contours);
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                if mask.is_border(x, y) {
                    let pixel = framebuffer.pixel(x, y);
                    assert!(![YELLOW, BLUE, RED, GREEN].contains(&pixel));
                }
            }
        }
    }
}
