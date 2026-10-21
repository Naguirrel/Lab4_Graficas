use std::{fs, path::Path};

#[derive(Clone, Copy, Debug)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
    pub fn sub(self, other: Self) -> Self {
        Self::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }
    pub fn dot(self, other: Self) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }
    pub fn cross(self, other: Self) -> Self {
        Self::new(
            self.y * other.z - self.z * other.y,
            self.z * other.x - self.x * other.z,
            self.x * other.y - self.y * other.x,
        )
    }
    pub fn normalized(self) -> Self {
        let length = self.dot(self).sqrt();
        Self::new(self.x / length, self.y / length, self.z / length)
    }
}

pub struct Mesh {
    pub vertices: Vec<Vec3>,
    pub triangles: Vec<[usize; 3]>,
}

impl Mesh {
    pub fn load(path: &Path) -> Result<Self, String> {
        let source =
            fs::read_to_string(path).map_err(|error| format!("{}: {error}", path.display()))?;
        Self::parse(&source)
    }

    pub fn parse(source: &str) -> Result<Self, String> {
        let mut mesh = Self {
            vertices: Vec::new(),
            triangles: Vec::new(),
        };
        for (line_number, line) in source.lines().enumerate() {
            let line = line.split('#').next().unwrap_or("").trim();
            let mut words = line.split_whitespace();
            match words.next() {
                Some("v") => {
                    let coords: Vec<_> = words.collect();
                    if coords.len() < 3 {
                        return Err(format!("linea {}: vertice incompleto", line_number + 1));
                    }
                    let parse = |s: &str| {
                        s.parse::<f32>()
                            .map_err(|_| format!("linea {}: coordenada invalida", line_number + 1))
                    };
                    let vertex = Vec3::new(parse(coords[0])?, parse(coords[1])?, parse(coords[2])?);
                    if !vertex.x.is_finite() || !vertex.y.is_finite() || !vertex.z.is_finite() {
                        return Err(format!("linea {}: coordenada no finita", line_number + 1));
                    }
                    mesh.vertices.push(vertex);
                }
                Some("f") => {
                    let mut face = Vec::new();
                    for reference in words {
                        face.push(
                            parse_reference(reference, mesh.vertices.len())
                                .map_err(|error| format!("linea {}: {error}", line_number + 1))?,
                        );
                    }
                    if face.first() == face.last() {
                        face.pop();
                    }
                    if face.len() < 3 {
                        return Err(format!(
                            "linea {}: cara con menos de tres vertices",
                            line_number + 1
                        ));
                    }
                    let triangles = triangulate(&face, &mesh.vertices)
                        .map_err(|error| format!("linea {}: {error}", line_number + 1))?;
                    mesh.triangles.extend(triangles);
                }
                _ => {} // vt, vn, mtllib, materiales y grupos no afectan las posiciones.
            }
        }
        if mesh.vertices.is_empty() || mesh.triangles.is_empty() {
            return Err("el OBJ no contiene vertices y caras dibujables".into());
        }
        Ok(mesh)
    }
}

fn parse_reference(reference: &str, count: usize) -> Result<usize, String> {
    let parts: Vec<_> = reference.split('/').collect();
    if parts.len() > 3
        || parts.is_empty()
        || parts[0].is_empty()
        || (parts.len() == 3 && parts[2].is_empty())
        || (parts.len() == 2 && parts[1].is_empty())
    {
        return Err(format!("referencia OBJ invalida: {reference}"));
    }
    for part in parts.iter().skip(1).filter(|part| !part.is_empty()) {
        let number = part
            .parse::<i64>()
            .map_err(|_| format!("referencia OBJ invalida: {reference}"))?;
        if number == 0 {
            return Err(format!("indice OBJ cero: {reference}"));
        }
    }
    let raw = parts[0]
        .parse::<i64>()
        .map_err(|_| format!("indice de vertice invalido: {reference}"))?;
    let index = if raw > 0 { raw - 1 } else { count as i64 + raw };
    if index < 0 || index >= count as i64 {
        return Err(format!("indice de vertice fuera de rango: {reference}"));
    }
    Ok(index as usize)
}

// Proyecta sobre el plano dominante y recorta orejas. Funciona tambien con
// caras concavas, donde un abanico desde el primer vertice daria area exterior.
fn triangulate(face: &[usize], vertices: &[Vec3]) -> Result<Vec<[usize; 3]>, String> {
    let mut normal = Vec3::new(0.0, 0.0, 0.0);
    for i in 0..face.len() {
        let a = vertices[face[i]];
        let b = vertices[face[(i + 1) % face.len()]];
        normal.x += (a.y - b.y) * (a.z + b.z);
        normal.y += (a.z - b.z) * (a.x + b.x);
        normal.z += (a.x - b.x) * (a.y + b.y);
    }
    let plane = if normal.x.abs() >= normal.y.abs() && normal.x.abs() >= normal.z.abs() {
        0
    } else if normal.y.abs() >= normal.z.abs() {
        1
    } else {
        2
    };
    let projected: Vec<(f32, f32)> = face
        .iter()
        .map(|&index| {
            let v = vertices[index];
            match plane {
                0 => (v.y, v.z),
                1 => (v.x, v.z),
                _ => (v.x, v.y),
            }
        })
        .collect();
    let cross = |a: usize, b: usize, c: usize| {
        let (ax, ay) = projected[a];
        let (bx, by) = projected[b];
        let (cx, cy) = projected[c];
        (bx - ax) * (cy - ay) - (by - ay) * (cx - ax)
    };
    let area: f32 = (0..face.len())
        .map(|i| {
            let (x, y) = projected[i];
            let (nx, ny) = projected[(i + 1) % face.len()];
            x * ny - nx * y
        })
        .sum();
    if area.abs() < 1e-8 {
        return Err("cara degenerada".into());
    }
    let sign = area.signum();
    let mut remaining: Vec<usize> = (0..face.len()).collect();
    let mut result = Vec::with_capacity(face.len() - 2);
    while remaining.len() > 3 {
        let mut ear = None;
        for i in 0..remaining.len() {
            let a = remaining[(i + remaining.len() - 1) % remaining.len()];
            let b = remaining[i];
            let c = remaining[(i + 1) % remaining.len()];
            if cross(a, b, c) * sign <= 1e-8 {
                continue;
            }
            let contains_other = remaining.iter().any(|&p| {
                p != a
                    && p != b
                    && p != c
                    && cross(a, b, p) * sign >= -1e-8
                    && cross(b, c, p) * sign >= -1e-8
                    && cross(c, a, p) * sign >= -1e-8
            });
            if !contains_other {
                ear = Some(i);
                break;
            }
        }
        let i = ear.ok_or("cara no triangulable (autointerseccion o degeneracion)")?;
        let a = remaining[(i + remaining.len() - 1) % remaining.len()];
        let b = remaining[i];
        let c = remaining[(i + 1) % remaining.len()];
        result.push([face[a], face[b], face[c]]);
        remaining.remove(i);
    }
    result.push([face[remaining[0]], face[remaining[1]], face[remaining[2]]]);
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_obj_reference_forms_and_negative_indices() {
        let mesh = Mesh::parse("v 0 0 0\nv 1 0 0\nv 0 1 0\nf 1 2 3\nf 1/1 2/2 3/3\nf -3//1 -2//1 -1//1\nf 1/1/1 2/2/1 3/3/1").unwrap();
        assert_eq!(mesh.triangles.len(), 4);
        assert!(mesh.triangles.iter().all(|t| {
            let mut v = *t;
            v.sort();
            v == [0, 1, 2]
        }));
        assert!(Mesh::parse("v 0 0 0\nv 1 0 0\nv 0 1 0\nf 1 2 4").is_err());
    }

    #[test]
    fn triangulates_concave_face_without_covering_notch() {
        let mesh =
            Mesh::parse("v 0 0 0\nv 3 0 0\nv 3 3 0\nv 1.5 1 0\nv 0 3 0\nf 1 2 3 4 5").unwrap();
        assert_eq!(mesh.triangles.len(), 3);
        let area: f32 = mesh
            .triangles
            .iter()
            .map(|t| {
                let a = mesh.vertices[t[0]];
                let b = mesh.vertices[t[1]];
                let c = mesh.vertices[t[2]];
                (b.x - a.x)
                    .mul_add(c.y - a.y, -(b.y - a.y) * (c.x - a.x))
                    .abs()
                    * 0.5
            })
            .sum();
        assert!((area - 6.0).abs() < 1e-5);
    }
}
