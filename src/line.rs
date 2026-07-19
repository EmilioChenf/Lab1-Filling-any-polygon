use crate::framebuffer::{Color, Framebuffer};
use crate::polygon::Point;

/// Bresenham general: cubre octantes, pendientes y direcciones arbitrarias.
pub fn draw_line_bresenham(framebuffer: &mut Framebuffer, start: Point, end: Point, color: Color) {
    visit_line_bresenham(start, end, |x, y| framebuffer.set_pixel(x, y, color));
}

/// Visita exactamente los mismos pixeles que usa Bresenham al dibujar.
pub fn visit_line_bresenham(start: Point, end: Point, mut visit: impl FnMut(i32, i32)) {
    let (mut x0, mut y0) = (start.x, start.y);
    let (x1, y1) = (end.x, end.y);
    let dx = (x1 - x0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let dy = -(y1 - y0).abs();
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut error = dx + dy;

    loop {
        visit(x0, y0);
        if x0 == x1 && y0 == y1 {
            break;
        }
        let twice_error = 2 * error;
        if twice_error >= dy {
            error += dy;
            x0 += sx;
        }
        if twice_error <= dx {
            error += dx;
            y0 += sy;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bresenham_incluye_ambos_extremos() {
        let black = Color::rgb(0, 0, 0);
        let white = Color::rgb(255, 255, 255);
        let mut framebuffer = Framebuffer::new(8, 8, black);
        draw_line_bresenham(&mut framebuffer, Point::new(6, 1), Point::new(1, 5), white);
        assert_eq!(framebuffer.pixel(6, 1), white);
        assert_eq!(framebuffer.pixel(1, 5), white);
    }

    #[test]
    fn bresenham_funciona_en_los_ocho_octantes() {
        let origin = Point::new(10, 10);
        for end in [
            Point::new(16, 12),
            Point::new(12, 16),
            Point::new(8, 16),
            Point::new(4, 12),
            Point::new(4, 8),
            Point::new(8, 4),
            Point::new(12, 4),
            Point::new(16, 8),
        ] {
            let mut visited = Vec::new();
            visit_line_bresenham(origin, end, |x, y| visited.push((x, y)));
            assert_eq!(visited.first(), Some(&(origin.x, origin.y)));
            assert_eq!(visited.last(), Some(&(end.x, end.y)));
            assert!(visited.windows(2).all(|pair| {
                (pair[1].0 - pair[0].0).abs() <= 1 && (pair[1].1 - pair[0].1).abs() <= 1
            }));
        }
    }
}
