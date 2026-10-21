use lab4_graficas::{
    framebuffer::Framebuffer,
    obj::{Mesh, Vec3},
    triangle::{ScreenVertex, paint_triangle, paint_triangle_edges},
};
use minifb::{Key, Window, WindowOptions};
use std::{env, path::PathBuf};

const WIDTH: usize = 900;
const HEIGHT: usize = 700;
const MODEL_COLOR: u32 = 0x00B7C9D5;
const EDGE_COLOR: u32 = 0x00374759;

fn render(mesh: &Mesh, fb: &mut Framebuffer) {
    // Eje Y del OBJ apunta arriba; en el framebuffer Y aumenta hacia abajo.
    let toward_viewer = Vec3::new(5.0, 7.0, 9.0).normalized();
    let right = Vec3::new(0.0, 1.0, 0.0).cross(toward_viewer).normalized();
    let up = toward_viewer.cross(right);
    let project = |v: Vec3| (v.dot(right), v.dot(up), v.dot(toward_viewer));
    let projected: Vec<_> = mesh.vertices.iter().copied().map(project).collect();
    let (mut min_x, mut max_x, mut min_y, mut max_y) = (
        f32::INFINITY,
        f32::NEG_INFINITY,
        f32::INFINITY,
        f32::NEG_INFINITY,
    );
    for &(x, y, _) in &projected {
        min_x = min_x.min(x);
        max_x = max_x.max(x);
        min_y = min_y.min(y);
        max_y = max_y.max(y);
    }
    let scale = (WIDTH as f32 * 0.84 / (max_x - min_x).max(1e-6))
        .min(HEIGHT as f32 * 0.84 / (max_y - min_y).max(1e-6));
    let center_x = (min_x + max_x) * 0.5;
    let center_y = (min_y + max_y) * 0.5;
    let screen: Vec<ScreenVertex> = projected
        .iter()
        .map(|&(x, y, depth)| ScreenVertex {
            x: WIDTH as f32 * 0.5 + (x - center_x) * scale,
            y: HEIGHT as f32 * 0.5 - (y - center_y) * scale,
            depth,
        })
        .collect();

    // Cada cara del OBJ ya fue convertida en indices de triangulos. Se recuperan
    // los tres vertices manualmente y se pasan al rasterizador por software.
    for &[a, b, c] in &mesh.triangles {
        paint_triangle(fb, [screen[a], screen[b], screen[c]], MODEL_COLOR);
    }
    // Los contornos facilitan leer las piezas sin agregar iluminacion.
    for &[a, b, c] in &mesh.triangles {
        paint_triangle_edges(fb, [screen[a], screen[b], screen[c]], EDGE_COLOR);
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut obj_path = PathBuf::from("Lab3.obj");
    let mut output_path = PathBuf::from("nave.png");
    let mut export_only = false;
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--obj" => obj_path = PathBuf::from(args.next().ok_or("falta ruta despues de --obj")?),
            "--output" => {
                output_path = PathBuf::from(args.next().ok_or("falta ruta despues de --output")?)
            }
            "--export-only" => export_only = true,
            "--help" | "-h" => {
                println!(
                    "Uso: cargo run -- [--obj ruta.obj] [--output imagen.png] [--export-only]"
                );
                return Ok(());
            }
            _ => return Err(format!("argumento desconocido: {arg}").into()),
        }
    }
    let mesh = Mesh::load(&obj_path)?;
    println!(
        "OBJ: {} vertices, {} triangulos",
        mesh.vertices.len(),
        mesh.triangles.len()
    );
    let mut fb = Framebuffer::new(WIDTH, HEIGHT);
    render(&mesh, &mut fb);
    fb.save_png(&output_path)?;
    println!("Imagen guardada en {}", output_path.display());
    if export_only {
        return Ok(());
    }

    let mut window = Window::new("Lab 4 - Nave OBJ", WIDTH, HEIGHT, WindowOptions::default())?;
    while window.is_open() && !window.is_key_down(Key::Escape) {
        window.update_with_buffer(&fb.pixels, WIDTH, HEIGHT)?;
    }
    Ok(())
}
