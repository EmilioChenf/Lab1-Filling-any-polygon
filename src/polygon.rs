use std::collections::VecDeque;

use crate::framebuffer::{Color, Framebuffer};
use crate::line::{draw_line_bresenham, visit_line_bresenham};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

/// Protege el contorno de Bresenham y distingue el interior rasterizado del fondo.
pub struct BorderMask {
    width: usize,
    height: usize,
    border: Vec<bool>,
    exterior: Vec<bool>,
}

impl BorderMask {
    pub fn is_border(&self, x: usize, y: usize) -> bool {
        self.border[y * self.width + x]
    }

    pub fn is_exterior(&self, x: usize, y: usize) -> bool {
        self.exterior[y * self.width + x]
    }

    pub fn is_interior(&self, x: usize, y: usize) -> bool {
        !self.is_border(x, y) && !self.is_exterior(x, y)
    }
}

/// Genera una mascara usando exactamente los pixeles que recorrera Bresenham.
pub fn build_outline_mask(width: usize, height: usize, contours: &[&[Point]]) -> BorderMask {
    let mut border = vec![false; width * height];
    for vertices in contours {
        if vertices.len() < 2 {
            continue;
        }
        for index in 0..vertices.len() {
            visit_line_bresenham(
                vertices[index],
                vertices[(index + 1) % vertices.len()],
                |x, y| {
                    if x >= 0 && y >= 0 && (x as usize) < width && (y as usize) < height {
                        border[y as usize * width + x as usize] = true;
                    }
                },
            );
        }
    }

    // Una inundacion 4-conexa desde los limites identifica todo pixel que queda
    // visualmente fuera de la barrera rasterizada por Bresenham.
    let mut exterior = vec![false; width * height];
    let mut queue = VecDeque::new();
    if width > 0 && height > 0 {
        for x in 0..width {
            enqueue_exterior_pixel(x, 0, width, &border, &mut exterior, &mut queue);
            enqueue_exterior_pixel(x, height - 1, width, &border, &mut exterior, &mut queue);
        }
        for y in 0..height {
            enqueue_exterior_pixel(0, y, width, &border, &mut exterior, &mut queue);
            enqueue_exterior_pixel(width - 1, y, width, &border, &mut exterior, &mut queue);
        }
    }

    while let Some((x, y)) = queue.pop_front() {
        if x > 0 {
            enqueue_exterior_pixel(x - 1, y, width, &border, &mut exterior, &mut queue);
        }
        if x + 1 < width {
            enqueue_exterior_pixel(x + 1, y, width, &border, &mut exterior, &mut queue);
        }
        if y > 0 {
            enqueue_exterior_pixel(x, y - 1, width, &border, &mut exterior, &mut queue);
        }
        if y + 1 < height {
            enqueue_exterior_pixel(x, y + 1, width, &border, &mut exterior, &mut queue);
        }
    }

    BorderMask {
        width,
        height,
        border,
        exterior,
    }
}

fn enqueue_exterior_pixel(
    x: usize,
    y: usize,
    width: usize,
    border: &[bool],
    exterior: &mut [bool],
    queue: &mut VecDeque<(usize, usize)>,
) {
    let index = y * width + x;
    if !border[index] && !exterior[index] {
        exterior[index] = true;
        queue.push_back((x, y));
    }
}

/// Relleno Scanline conservador, protegido por la rasterizacion de los contornos.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FillReport {
    pub total_fill_writes: usize,
    pub fill_writes_inside_holes: usize,
}

pub fn fill_polygon_scanline(
    framebuffer: &mut Framebuffer,
    vertices: &[Point],
    holes: &[(&[Point], &BorderMask)],
    color: Color,
    mask: &BorderMask,
) -> FillReport {
    let mut report = FillReport::default();
    if vertices.len() < 3 {
        return report;
    }
    assert_eq!(framebuffer.width(), mask.width);
    assert_eq!(framebuffer.height(), mask.height);

    let min_y = vertices.iter().map(|point| point.y).min().unwrap_or(0);
    let max_y = vertices.iter().map(|point| point.y).max().unwrap_or(0);

    for y in min_y.max(0)..max_y.min(framebuffer.height() as i32) {
        let scan_y = y as f64 + 0.5;
        let mut scanline_intersections = Vec::with_capacity(vertices.len());

        for index in 0..vertices.len() {
            let first = vertices[index];
            let second = vertices[(index + 1) % vertices.len()];
            if first.y == second.y {
                continue;
            }
            let lower_y = first.y.min(second.y) as f64;
            let upper_y = first.y.max(second.y) as f64;
            if scan_y >= lower_y && scan_y < upper_y {
                let factor = (scan_y - first.y as f64) / (second.y - first.y) as f64;
                scanline_intersections.push(first.x as f64 + factor * (second.x - first.x) as f64);
            }
        }

        scanline_intersections.sort_by(|a, b| a.total_cmp(b));
        for pair in scanline_intersections.chunks_exact(2) {
            // x + 0.5 > izquierda  => x > izquierda - 0.5
            // x + 0.5 < derecha    => x < derecha - 0.5
            let start_x = ((pair[0] - 0.5).floor() as i32 + 1).max(0);
            let end_x = ((pair[1] - 0.5).ceil() as i32).min(framebuffer.width() as i32);
            for x in start_x..end_x {
                let center_x = x as f64 + 0.5;
                if center_x <= pair[0] || center_x >= pair[1] {
                    continue;
                }
                let ux = x as usize;
                let uy = y as usize;
                if mask.is_border(ux, uy) || mask.is_exterior(ux, uy) {
                    continue;
                }
                if holes.iter().any(|(hole, hole_mask)| {
                    hole_mask.is_border(ux, uy)
                        || hole_mask.is_interior(ux, uy)
                        || is_point_strictly_inside_polygon(center_x, scan_y, hole)
                }) {
                    continue;
                }
                // Verificacion independiente del intervalo Scanline para cada candidato.
                if is_point_strictly_inside_polygon(center_x, scan_y, vertices) {
                    framebuffer.set_pixel(x, y, color);
                    report.total_fill_writes += 1;
                    if holes
                        .iter()
                        .any(|(_, hole_mask)| hole_mask.is_interior(ux, uy))
                    {
                        report.fill_writes_inside_holes += 1;
                    }
                }
            }
        }
    }

    // Bresenham es la autoridad visual final. Scanline realiza el relleno
    // geometrico y esta fase completa toda celda encerrada por su frontera
    // rasterizada que la prueba continua haya descartado junto a una diagonal.
    for y in 0..framebuffer.height() {
        for x in 0..framebuffer.width() {
            if mask.is_interior(x, y)
                && !holes
                    .iter()
                    .any(|(_, hole_mask)| hole_mask.is_border(x, y) || hole_mask.is_interior(x, y))
            {
                framebuffer.set_pixel(x as i32, y as i32, color);
                report.total_fill_writes += 1;
                if holes
                    .iter()
                    .any(|(_, hole_mask)| hole_mask.is_interior(x, y))
                {
                    report.fill_writes_inside_holes += 1;
                }
            }
        }
    }
    report
}

/// Regla par-impar; devuelve false para cualquier punto situado sobre una arista.
pub fn is_point_strictly_inside_polygon(x: f64, y: f64, vertices: &[Point]) -> bool {
    if vertices.len() < 3 {
        return false;
    }
    let mut inside = false;
    for index in 0..vertices.len() {
        let a = vertices[index];
        let b = vertices[(index + 1) % vertices.len()];
        let (ax, ay) = (a.x as f64, a.y as f64);
        let (bx, by) = (b.x as f64, b.y as f64);
        let cross = (x - ax) * (by - ay) - (y - ay) * (bx - ax);
        let on_segment = cross.abs() < 1.0e-9
            && x >= ax.min(bx)
            && x <= ax.max(bx)
            && y >= ay.min(by)
            && y <= ay.max(by);
        if on_segment {
            return false;
        }
        if (ay > y) != (by > y) {
            let intersection_x = ax + (y - ay) * (bx - ax) / (by - ay);
            if intersection_x > x {
                inside = !inside;
            }
        }
    }
    inside
}

pub fn draw_polygon_outline(framebuffer: &mut Framebuffer, vertices: &[Point], color: Color) {
    if vertices.len() < 2 {
        return;
    }
    for index in 0..vertices.len() {
        draw_line_bresenham(
            framebuffer,
            vertices[index],
            vertices[(index + 1) % vertices.len()],
            color,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn punto_en_borde_no_es_interior() {
        let square = [
            Point::new(2, 2),
            Point::new(8, 2),
            Point::new(8, 8),
            Point::new(2, 8),
        ];
        assert!(is_point_strictly_inside_polygon(3.5, 3.5, &square));
        assert!(!is_point_strictly_inside_polygon(2.0, 4.0, &square));
        assert!(!is_point_strictly_inside_polygon(1.5, 4.5, &square));
    }

    #[test]
    fn mascara_impide_rellenar_borde_y_exterior_rasterizado() {
        let background = Color::rgb(1, 2, 3);
        let fill = Color::rgb(9, 8, 7);
        let white = Color::rgb(255, 255, 255);
        let polygon = [Point::new(2, 9), Point::new(6, 2), Point::new(10, 9)];
        let mut framebuffer = Framebuffer::new(13, 12, background);
        let mask = build_outline_mask(13, 12, &[&polygon]);
        fill_polygon_scanline(&mut framebuffer, &polygon, &[], fill, &mask);
        draw_polygon_outline(&mut framebuffer, &polygon, white);

        for y in 0..12 {
            for x in 0..13 {
                if mask.is_border(x, y) {
                    assert_eq!(framebuffer.pixel(x, y), white);
                }
                if framebuffer.pixel(x, y) == fill {
                    assert!(is_point_strictly_inside_polygon(
                        x as f64 + 0.5,
                        y as f64 + 0.5,
                        &polygon
                    ));
                    assert!(!mask.is_exterior(x, y));
                }
            }
        }
        assert_eq!(framebuffer.pixel(6, 5), fill);
        assert_eq!(framebuffer.pixel(1, 8), background);
        assert_eq!(framebuffer.pixel(11, 8), background);
    }

    #[test]
    fn scanline_rellena_poligonos_convexos_y_concavos() {
        let background = Color::rgb(0, 0, 0);
        let fill = Color::rgb(10, 20, 30);
        for (polygon, interior_sample) in [
            (
                vec![
                    Point::new(2, 2),
                    Point::new(9, 2),
                    Point::new(9, 9),
                    Point::new(2, 9),
                ],
                (5, 5),
            ),
            (
                vec![
                    Point::new(2, 2),
                    Point::new(10, 2),
                    Point::new(10, 5),
                    Point::new(6, 5),
                    Point::new(6, 10),
                    Point::new(2, 10),
                ],
                (4, 7),
            ),
        ] {
            let mut framebuffer = Framebuffer::new(13, 13, background);
            let mask = build_outline_mask(13, 13, &[&polygon]);
            let report = fill_polygon_scanline(&mut framebuffer, &polygon, &[], fill, &mask);
            assert!(report.total_fill_writes > 0);
            assert_eq!(
                framebuffer.pixel(interior_sample.0, interior_sample.1),
                fill
            );
        }
    }
}
