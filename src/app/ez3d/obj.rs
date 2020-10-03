use super::*;

pub struct Obj {
    vb: ugli::VertexBuffer<Vertex>,
}

impl Obj {
    pub fn vb(&self) -> &ugli::VertexBuffer<Vertex> {
        &self.vb
    }
}

impl geng::LoadAsset for Obj {
    const DEFAULT_EXT: Option<&'static str> = None;
    fn load(geng: &Rc<Geng>, path: &str) -> geng::AssetFuture<Self> {
        let geng = geng.clone();
        let path = path.to_owned();
        Box::pin(async move {
            let obj_source =
                <String as geng::LoadAsset>::load(&geng, &format!("{}.obj", path)).await?;
            let mtl_source =
                <String as geng::LoadAsset>::load(&geng, &format!("{}.mtl", path)).await?;
            let mut v = Vec::new();
            let mut vn = Vec::new();
            let mut vt = Vec::new();
            let mut mesh = Vec::new();
            let mut color = Color::WHITE;
            let mut current_material = String::new();
            let mut materials = HashMap::<String, Color<f32>>::new();
            for line in mtl_source.lines().chain(obj_source.lines()) {
                if line.starts_with("v ") {
                    let mut parts = line.split_whitespace();
                    parts.next();
                    let x: f32 = parts.next().unwrap().parse().unwrap();
                    let y: f32 = parts.next().unwrap().parse().unwrap();
                    let z: f32 = parts.next().unwrap().parse().unwrap();
                    v.push(vec3(x, y, z));
                } else if line.starts_with("vn ") {
                    let mut parts = line.split_whitespace();
                    parts.next();
                    let x: f32 = parts.next().unwrap().parse().unwrap();
                    let y: f32 = parts.next().unwrap().parse().unwrap();
                    let z: f32 = parts.next().unwrap().parse().unwrap();
                    vn.push(vec3(x, y, z));
                } else if line.starts_with("vt ") {
                    let mut parts = line.split_whitespace();
                    parts.next();
                    let x: f32 = parts.next().unwrap().parse().unwrap();
                    let y: f32 = parts.next().unwrap().parse().unwrap();
                    vt.push(vec2(x, y));
                } else if line.starts_with("f ") {
                    let mut parts = line.split_whitespace();
                    parts.next();
                    let to_vertex = |s: &str| {
                        let mut parts = s.split("/");
                        let i_v: usize = parts.next().unwrap().parse().unwrap();
                        let i_vt: usize = parts.next().unwrap().parse().unwrap();
                        let i_vn: usize = parts.next().unwrap().parse().unwrap();
                        Vertex {
                            a_pos: v[i_v - 1],
                            a_color: color,
                            // a_vn: vn[i_vn - 1],
                            // a_vt: vt[i_vt - 1],
                        }
                    };
                    let mut cur = Vec::new();
                    while let Some(s) = parts.next() {
                        cur.push(to_vertex(s));
                    }
                    for i in 2..cur.len() {
                        mesh.push(cur[0].clone());
                        mesh.push(cur[i - 1].clone());
                        mesh.push(cur[i].clone());
                    }
                } else if line.starts_with("usemtl ") {
                    let material = line[6..].trim();
                    color = materials[material];
                } else if line.starts_with("newmtl ") {
                    current_material = line[6..].trim().to_owned();
                    materials.insert(current_material.clone(), Color::WHITE);
                } else if line.starts_with("Kd ") {
                    let mut parts = line.split_whitespace();
                    parts.next();
                    *materials.get_mut(&current_material).unwrap() = Color::rgb(
                        parts.next().unwrap().parse().unwrap(),
                        parts.next().unwrap().parse().unwrap(),
                        parts.next().unwrap().parse().unwrap(),
                    );
                }
            }
            for vertex in &mut mesh {
                fn fix(v: Vec3<f32>) -> Vec3<f32> {
                    vec3(v.x, v.z, v.y)
                }
                vertex.a_pos = fix(vertex.a_pos);
            }
            Ok(Self {
                vb: ugli::VertexBuffer::new_static(geng.ugli(), mesh),
            })
        })
    }
}
