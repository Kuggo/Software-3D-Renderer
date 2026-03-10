use crate::{Camera, Screen};
use crate::geometry::*;
use crate::geometry::Triangle;
use crate::utils::{fp_equals, Color, Pixel, Vec2, Vec3};

#[derive(Debug, Copy, Clone)]
pub enum InterpMode {
    Linear,
    PerspectiveCorrect,
}

#[derive(Debug, Copy, Clone)]
pub enum RenderMode {
    Wireframe,
    Solid,
}

#[derive(Debug, Copy, Clone)]
pub enum CullMode {
    None,
    Backface,
    Frontface,
}


impl Triangle {
    /// Returns the linear interpolation of the triangle's vertices by barycentric coordinates `alpha` and `beta`.
    pub fn tri_lerp(&self, alpha: f32, beta: f32) -> Vertex {
        let pos = Vec3::tri_lerp(&self.a.pos, &self.b.pos, &self.c.pos, alpha, beta);
        let color = Color::tri_lerp(&self.a.color, &self.b.color, &self.c.color, alpha, beta);
        Vertex { pos, color }
    }

    pub fn ztri_lerp(&self, alpha: f32, beta: f32) -> Vertex {
        let pos = Vec3::ztri_lerp(&self.a.pos, &self.b.pos, &self.c.pos, alpha, beta);
        let color = Color::tri_lerp(
            &self.a.color.scale(1.0/&self.a.pos.z),
            &self.b.color.scale(1.0/&self.b.pos.z),
            &self.c.color.scale(1.0/&self.c.pos.z),
            alpha, beta).scale(pos.z);
        Vertex {
            pos,
            color,
        }
    }

    /// Computes the barycentric coordinates of a point `p` with respect to the triangle.
    pub fn barycentric_coords(p: Pixel, pa: Pixel, pb: Pixel, pc: Pixel) -> Option<(f32, f32, f32)> {
        let v0 = Vec2::new((pb.x - pa.x) as f32, (pb.y - pa.y) as f32);
        let v1 = Vec2::new((pc.x - pa.x) as f32, (pc.y - pa.y) as f32);
        let v2 = Vec2::new((p.x - pa.x) as f32, (p.y - pa.y) as f32);

        let d00 = v0.dot(&v0);
        let d01 = v0.dot(&v1);
        let d11 = v1.dot(&v1);

        let denom = d00 * d11 - d01 * d01;
        if fp_equals(denom, 0.0) {
            return None; // Degenerate triangle
        }

        let d20 = v2.dot(&v0);
        let d21 = v2.dot(&v1);

        let inv_denom = 1.0 / denom;
        let v = (d11 * d20 - d01 * d21) * inv_denom;
        let w = (d00 * d21 - d01 * d20) * inv_denom;
        let u = 1.0 - v - w;

        Some((u, v, w))
    }
}

impl Vec3 {
    /// Linearly interpolate between three Vec3 (a, b, and c) using barycentric coordinates (u and v).
    pub fn tri_lerp(a: &Vec3, b: &Vec3, c: &Vec3, u: f32, v: f32) -> Vec3 {
        let w = 1.0 - u - v;
        Vec3 {
            x: a.x * w + b.x * u + c.x * v,
            y: a.y * w + b.y * u + c.y * v,
            z: a.z * w + b.z * u + c.z * v,
        }
    }

    /// Perspective-correctly interpolate between three Vec3 (a, b, and c) using barycentric coordinates (u and v).
    pub fn ztri_lerp(a: &Vec3, b: &Vec3, c: &Vec3, alpha: f32, beta: f32) -> Vec3 {
        let gamma = 1.0 - alpha - beta;
        let inv_za = 1.0 / a.z;
        let inv_zb = 1.0 / b.z;
        let inv_zc = 1.0 / c.z;

        let z = 1.0 / (inv_za * gamma + inv_zb * alpha + inv_zc * beta);

        let x = (a.x * inv_za * gamma + b.x * inv_zb * alpha + c.x * inv_zc * beta) * z;
        let y = (a.y * inv_za * gamma + b.y * inv_zb * alpha + c.y * inv_zc * beta) * z;

        Vec3 { x, y, z }
    }
}

impl Color {
    /// Linearly interpolate between three colors `c0`, `c1`, and `c2` by weights `w0`, `w1`, and `w2`.
    pub fn tri_lerp(c0: &Color, c1: &Color, c2: &Color, alpha: f32, beta: f32) -> Color {
        let gamma = 1.0 - alpha - beta;
        Color {
            r: c0.r * alpha + c1.r * beta + c2.r * gamma,
            g: c0.g * alpha + c1.g * beta + c2.g * gamma,
            b: c0.b * alpha + c1.b * beta + c2.b * gamma,
            a: c0.a * alpha + c1.a * beta + c2.a * gamma,
        }
    }
}

impl Vertex {
    /// Returns the perspective-correct linear interpolation between two vertices `a` and `b` by a factor `t`.
    /// This is used to interpolate vertex attributes in screen space after perspective projection, to avoid distortion.
    /// t should be in the range [0.0, 1.0].
    pub fn zlerp(a: &Vertex, b: &Vertex, t: f32) -> Vertex {
        let z = 1.0 / lerp(1.0/a.pos.z, 1.0/b.pos.z, t);

        let pos = Vec3::lerp(
            &a.pos.scale(1.0/a.pos.z), &b.pos.scale(1.0/a.pos.z), t
        ).scale(z);

        let color = Color::lerp(
            &a.color, &b.color, t
        ).scale(z);

        Vertex { pos, color }
    }
}



/// The Renderer is responsible for taking a Scene and a Camera, and producing a 2D image on the Screen.
/// Right-handed
/// X+ -> right
/// Y+ -> up
/// Z+ -> forward
pub struct Renderer<'a> {
    focal_length: f32,
    pub screen: &'a mut Screen,
    interpolation_mode: InterpMode,
    cull_mode: CullMode,
    render_mode: RenderMode,
}
impl<'a> Renderer<'a> {
    /// Creates a new Renderer with a reference to the Screen it will draw on.
    pub fn new_to_screen(
        screen: &'a mut Screen,
        interpolation_mode: InterpMode, cull_mode: CullMode, render_mode: RenderMode
    ) -> Self {
        Renderer { focal_length: 1.0, screen, interpolation_mode, cull_mode, render_mode }
    }

    /// Renders the scene from the perspective of the given camera onto `self.screen`.
    pub fn render_scene_from_camera(&mut self, camera: &Camera) {
        let view = Transform::inverse(&camera.transform);
        self.focal_length = camera.get_focal_length(self.screen.get_width_units());
        for object in &camera.scene.objects {
            self.render_object(object, &view);
        }
    }

    /// Renders a single object by applying the view transformation and then rasterizing its primitives.
    fn render_object(&mut self, object: &Object, view: &Transform) {
        let combined_transform = view.combine_with(&object.transform);
        for primitive in &object.mesh.primitives {
            match primitive {
                Primitive::Triangle(t) => self.render_triangle(t, &combined_transform),
                Primitive::Line(v1, v2) => self.render_line(v1, v2, &combined_transform),
                Primitive::Point(v) => self.render_point(v, &combined_transform),
            }
        }
    }

    /// Renders a triangle by drawing the color of every pixel it covers onto the screen.
    /// It starts by applying the combined view and object transformations to the triangle vertices,
    /// then projects them to 2D screen space, and finally converts those to pixel coordinates.
    /// Every pixel in the triangle should be colored according to their interpolated attributes.
    fn render_triangle(&mut self, triangle: &Triangle, transform: &Transform) {
        // Apply transformations to the triangle vertices
        let a = transform.apply_to(&triangle.a.pos);
        let b = transform.apply_to(&triangle.b.pos);
        let c = transform.apply_to(&triangle.c.pos);

        // Project the vertices to 2D screen space
        let sa = self.perspective_project(&a);
        let sb = self.perspective_project(&b);
        let sc = self.perspective_project(&c);

        // we are assuming a CCW winding order of ABC
        let front_facing = (sb.x - sa.x) * (sc.y - sa.y) - (sb.y - sa.y) * (sc.x - sa.x) < 0.0;
        let cull = match self.cull_mode {
            CullMode::None => false,
            CullMode::Backface => !front_facing,
            CullMode::Frontface => front_facing,
        };
        if cull { return; }

        // convert from screen space to pixel coordinates
        let pa = self.screen.world_to_screen_coords(sa);
        let pb = self.screen.world_to_screen_coords(sb);
        let pc = self.screen.world_to_screen_coords(sc);

        match self.render_mode {
            RenderMode::Solid => self.rasterize_triangle_fast(pa, pb, pc, triangle),
            RenderMode::Wireframe => {
                self.rasterize_line(pa, pb, &triangle.a, &triangle.b);
                self.rasterize_line(pb, pc, &triangle.b, &triangle.c);
                self.rasterize_line(pc, pa, &triangle.c, &triangle.a);
            }
        }
    }

    /// Renders a line by drawing the color of every pixel it covers onto the screen.
    /// It starts by applying the combined view and object transformations to the line endpoints,
    /// then projects them to 2D screen space, and finally converts those to pixel coordinates.
    /// Every pixel in the line should be colored according to their interpolated attributes.
    fn render_line(&mut self, v1: &Vertex, v2: &Vertex, transform: &Transform) {
        let a = transform.apply_to(&v1.pos);
        let b = transform.apply_to(&v2.pos);

        let sa = self.perspective_project(&a);
        let sb = self.perspective_project(&b);

        let pa = self.screen.world_to_screen_coords(sa);
        let pb = self.screen.world_to_screen_coords(sb);

        self.rasterize_line(pa, pb, v1, v2);
    }

    /// Renders a point by drawing its color in its calculated screen position.
    /// This calculation starts by applying the combined view and object transformations to it,
    /// then projects it to 2D screen space, and finally converts that to pixel coordinates.
    fn render_point(&mut self, vertex: &Vertex, transform: &Transform) {
        let v = transform.apply_to(&vertex.pos);
        let sv = self.perspective_project(&v);
        let pv = self.screen.world_to_screen_coords(sv);
        self.screen.draw_pixel(pv, &vertex.color);
    }

    /// Rasterizes a triangle by filling in the color of every pixel it covers onto the screen.
    fn rasterize_triangle(&mut self, pa: Pixel, pb: Pixel, pc: Pixel, triangle: &Triangle) {
        let bottom = pa.y.min(pb.y).min(pc.y);
        let top = pa.y.max(pb.y).max(pc.y);
        let left = pa.x.min(pb.x).min(pc.x);
        let right = pa.x.max(pb.x).max(pc.x);

        for y in bottom..=top {
            for x in left..=right {
                let p = Pixel::new(x, y);
                if let Some((u, v, w)) = Triangle::barycentric_coords(p, pa, pb, pc) {
                    if u >= 0.0 && v >= 0.0 && w >= 0.0 {
                        let pixel_attributes = match self.interpolation_mode {
                            InterpMode::Linear => triangle.tri_lerp(u, v),
                            InterpMode::PerspectiveCorrect => triangle.ztri_lerp(u, v)
                        };
                        self.screen.draw_pixel(p, &pixel_attributes.color);
                    }
                }
            }
        }
    }

    /// Rasterizes a triangle by filling in the color of every pixel it covers onto the screen.
    /// It precomputes triangle constants to make the inner loop faster, avoiding the overhead of
    /// barycentric_coords function.
    fn rasterize_triangle_fast(&mut self, pa: Pixel, pb: Pixel, pc: Pixel, triangle: &Triangle) {
        let bottom = pa.y.min(pb.y).min(pc.y);
        let top = pa.y.max(pb.y).max(pc.y);
        let left = pa.x.min(pb.x).min(pc.x);
        let right = pa.x.max(pb.x).max(pc.x);

        let v0 = Vec2::new((pb.x - pa.x) as f32, (pb.y - pa.y) as f32);
        let v1 = Vec2::new((pc.x - pa.x) as f32, (pc.y - pa.y) as f32);
        let d00 = v0.dot(&v0);
        let d01 = v0.dot(&v1);
        let d11 = v1.dot(&v1);

        let denom = d00 * d11 - d01 * d01;
        if fp_equals(denom, 0.0) {
            return; // Degenerate triangle aka no area, so we can skip rasterization
        }
        let inv_denom = 1.0 / denom;

        for y in bottom..=top {
            for x in left..=right {
                let v2 = Vec2::new((x - pa.x) as f32, (y - pa.y) as f32);
                let d20 = v2.dot(&v0);
                let d21 = v2.dot(&v1);
                let v = (d11 * d20 - d01 * d21) * inv_denom;
                let w = (d00 * d21 - d01 * d20) * inv_denom;
                let u = 1.0 - v - w;
                if u >= 0.0 && v >= 0.0 && w >= 0.0 {
                    let pixel_attributes = match self.interpolation_mode {
                        InterpMode::Linear => triangle.tri_lerp(u, v),
                        InterpMode::PerspectiveCorrect => triangle.ztri_lerp(u, v)
                    };
                    self.screen.draw_pixel(Pixel::new(x, y), &pixel_attributes.color);
                }
            }
        }
    }

    /// Rasterizes a line by filling in the color of every pixel it covers onto the screen.
    fn rasterize_line(&mut self, p1: Pixel, p2: Pixel, v1: &Vertex, v2: &Vertex) {
        let dx = p2.x - p1.x;
        let dy = p2.y - p1.y;
        if dx == 0 && dy == 0 {
            self.screen.draw_pixel(p1, &v1.color);
            return;
        }

        // this closure can calculate the color at any point by interpolating between the two points
        let interpolate_color =|t| {
            match self.interpolation_mode {
                InterpMode::Linear => Vertex::lerp(v1, v2, t).color,
                InterpMode::PerspectiveCorrect => Vertex::zlerp(v1, v2, t).color
            }
        };

        if dy == 0 {    // screen can already handle horizontal spans, so we can optimize for that case
            self.screen.draw_h_line(p1.y, p1.x, p2.x, interpolate_color);
            return;
        }
        // vertical spans are not supported by the screen, so we just use the general case

        let steps = dx.abs().max(dy.abs());
        let x_inc = dx as f32 / steps as f32;
        let y_inc = dy as f32 / steps as f32;

        let mut pos = Vec2::new(p1.x as f32, p1.y as f32);
        for i in 0..=steps {
            let t = i as f32 / steps as f32;
            self.screen.draw_pixel(Pixel::from_vec2(pos), &interpolate_color(t));
            pos.x += x_inc;
            pos.y += y_inc;
        }
    }

    /// Projects a 3D point in camera space to 2D screen space using perspective projection.
    fn perspective_project(&self, point: &Vec3) -> Vec2 {
        Vec2::new(point.x, point.y).scale(self.focal_length/point.z)
    }
}