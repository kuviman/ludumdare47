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
    const DEFAULT_EXT: Option<&'static str> = Some("obj");
    fn load(geng: &Rc<Geng>, path: &str) -> geng::AssetFuture<Self> {
        let geng = geng.clone();
        let path = path.to_owned();
        Box::pin(async move {
            let dir = match path.rfind('/') {
                Some(index) => &path[..index],
                None => ".",
            };
            let obj_source = <String as geng::LoadAsset>::load(&geng, &path).await?;
            let mut v = Vec::new();
            let mut vn = Vec::new();
            let mut vt = Vec::new();
            let mut mesh = Vec::new();
            let mut current_material = String::new();
            #[derive(Clone)]
            struct Material {
                color: Color<f32>,
                emission: f32,
            }
            impl Material {
                fn new() -> Self {
                    Self {
                        color: Color::WHITE,
                        emission: 0.0,
                    }
                }
            }
            let mut material = Material::new();
            let mut materials = HashMap::<String, Material>::new();
            for line in obj_source.lines() {
                if line.starts_with("mtllib ") {
                    let mut parts = line.split_whitespace();
                    parts.next();
                    let mtl_source = <String as geng::LoadAsset>::load(
                        &geng,
                        &format!("{}/{}", dir, parts.next().unwrap()),
                    )
                    .await?;
                    for line in mtl_source.lines() {
                        if line.starts_with("newmtl ") {
                            current_material = line[6..].trim().to_owned();
                            materials.insert(current_material.clone(), Material::new());
                        } else if line.starts_with("Kd ") {
                            let mut parts = line.split_whitespace();
                            parts.next();
                            materials.get_mut(&current_material).unwrap().color.r =
                                parts.next().unwrap().parse().unwrap();
                            materials.get_mut(&current_material).unwrap().color.g =
                                parts.next().unwrap().parse().unwrap();
                            materials.get_mut(&current_material).unwrap().color.b =
                                parts.next().unwrap().parse().unwrap();
                        } else if line.starts_with("Ke ") {
                            let mut parts = line.split_whitespace();
                            parts.next();
                            materials.get_mut(&current_material).unwrap().emission =
                                parts.next().unwrap().parse().unwrap();
                        }
                    }
                }
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
                        let _i_vt: usize = parts.next().unwrap().parse().unwrap();
                        let _i_vn: usize = parts.next().unwrap().parse().unwrap();
                        Vertex {
                            a_pos: v[i_v - 1],
                            a_normal: vec3(0.0, 0.0, 0.0),
                            a_color: material.color,
                            a_emission: material.emission,
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
                    material = materials[line[6..].trim()].clone();
                }
            }
            for vertex in &mut mesh {
                fn fix(v: Vec3<f32>) -> Vec3<f32> {
                    vec3(v.x, -v.z, v.y)
                }
                vertex.a_pos = fix(vertex.a_pos);
            }
            calc_normals(&mut mesh);
            Ok(Self {
                vb: ugli::VertexBuffer::new_static(geng.ugli(), mesh),
            })
        })
    }
}
