use crate::framebuffer::Framebuffer;

#[derive(Clone, Copy)]
pub struct ScreenVertex {
    pub x: f32,
    pub y: f32,
    pub depth: f32,
}

fn edge(a: ScreenVertex, b: ScreenVertex, x: f32, y: f32) -> f32 {
    (x - a.x) * (b.y - a.y) - (y - a.y) * (b.x - a.x)
}

// Rasterizacion por software: baricentros + prueba de profundidad por pixel.
pub fn paint_triangle(fb: &mut Framebuffer, vertices: [ScreenVertex; 3], color: u32) {
    let [a, b, c] = vertices;
    let area = edge(a, b, c.x, c.y);
    if area.abs() < 1e-6 {
        return;
    }
    let min_x = a.x.min(b.x).min(c.x).floor().max(0.0) as i32;
    let max_x = a.x.max(b.x).max(c.x).ceil().min((fb.width - 1) as f32) as i32;
    let min_y = a.y.min(b.y).min(c.y).floor().max(0.0) as i32;
    let max_y = a.y.max(b.y).max(c.y).ceil().min((fb.height - 1) as f32) as i32;
    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let px = x as f32 + 0.5;
            let py = y as f32 + 0.5;
            let w0 = edge(b, c, px, py) / area;
            let w1 = edge(c, a, px, py) / area;
            let w2 = edge(a, b, px, py) / area;
            if w0 >= -1e-5 && w1 >= -1e-5 && w2 >= -1e-5 {
                fb.point(x, y, w0 * a.depth + w1 * b.depth + w2 * c.depth, color);
            }
        }
    }
}

pub fn paint_triangle_edges(fb: &mut Framebuffer, vertices: [ScreenVertex; 3], color: u32) {
    for i in 0..3 {
        let a = vertices[i];
        let b = vertices[(i + 1) % 3];
        let steps = (b.x - a.x).abs().max((b.y - a.y).abs()).ceil() as i32;
        for step in 0..=steps {
            let t = if steps == 0 {
                0.0
            } else {
                step as f32 / steps as f32
            };
            fb.edge_point(
                (a.x + (b.x - a.x) * t).round() as i32,
                (a.y + (b.y - a.y) * t).round() as i32,
                a.depth + (b.depth - a.depth) * t,
                color,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn closer_triangle_wins_independent_of_draw_order() {
        let mut fb = Framebuffer::new(20, 20);
        let verts = |depth| {
            [
                ScreenVertex {
                    x: 2.0,
                    y: 2.0,
                    depth,
                },
                ScreenVertex {
                    x: 17.0,
                    y: 2.0,
                    depth,
                },
                ScreenVertex {
                    x: 2.0,
                    y: 17.0,
                    depth,
                },
            ]
        };
        paint_triangle(&mut fb, verts(2.0), 0xFFFFFF);
        paint_triangle(&mut fb, verts(1.0), 0x000000);
        assert_eq!(fb.pixels[5 * 20 + 5], 0xFFFFFF);
    }
}
