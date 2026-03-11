use crate::{Camera, Screen};
use crate::geometry::*;
use crate::geometry::Triangle;
use crate::utils::{fp_equals, Color, Pixel, Vec2, Vec3, FP_TOLERANCE};

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

#[derive(Debug, Copy, Clone)]
pub enum DepthTest {
    None,
    Fail,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    Equal,
    NotEqual,
}
impl DepthTest {
    const DONT_CARE: f32 = 0.0;
    pub fn test(&self, a: f32, b: f32) -> bool {
        match self {
            DepthTest::None => true,
            DepthTest::Fail => false,
            DepthTest::Less => a < b,
            DepthTest::LessEqual => a <= b,
            DepthTest::Greater => a > b,
            DepthTest::GreaterEqual => a >= b,
            DepthTest::Equal => fp_equals(a, b),
            DepthTest::NotEqual => !fp_equals(a, b),
        }
    }

    pub fn starting_value(&self) -> f32 {
        match self {
            DepthTest::None => Self::DONT_CARE,
            DepthTest::Fail => Self::DONT_CARE,
            DepthTest::Less => f32::INFINITY,
            DepthTest::LessEqual => f32::INFINITY,
            DepthTest::Greater => f32::NEG_INFINITY,
            DepthTest::GreaterEqual => f32::NEG_INFINITY,
            DepthTest::Equal => Self::DONT_CARE,
            DepthTest::NotEqual => Self::DONT_CARE,
        }
    }
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
            x: a.x * u + b.x * v + c.x * w,
            y: a.y * u + b.y * v + c.y * w,
            z: a.z * u + b.z * v + c.z * w,
        }
    }

    /// Perspective-correctly interpolate between three Vec3 (a, b, and c) using barycentric coordinates (u and v).
    pub fn ztri_lerp(a: &Vec3, b: &Vec3, c: &Vec3, alpha: f32, beta: f32) -> Vec3 {
        let gamma = 1.0 - alpha - beta;
        let inv_za_weight = alpha / a.z;
        let inv_zb_weight = beta / b.z;
        let inv_zc_weight = gamma / c.z;

        let z = 1.0 / (inv_za_weight + inv_zb_weight + inv_zc_weight);

        let x = (a.x * inv_za_weight + b.x * inv_zb_weight + c.x * inv_zc_weight) * z;
        let y = (a.y * inv_za_weight + b.y * inv_zb_weight + c.y * inv_zc_weight) * z;

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
            &a.pos.scale(1.0/a.pos.z), &b.pos.scale(1.0/b.pos.z), t
        ).scale(z);

        let color = Color::lerp(
            &a.color.scale(1.0/a.pos.z), &b.color.scale(1.0/b.pos.z), t
        ).scale(z);

        Vertex { pos, color }
    }
}



struct RenderingContext<'a> {
    screen: &'a mut Screen,
    transform: &'a Transform,
    shader: &'a dyn Fn(&Vertex) -> Color,
}


/// The Renderer is responsible for taking a Scene and a Camera, and producing a 2D image on the Screen.
/// Right-handed
/// X+ -> right
/// Y+ -> up
/// Z+ -> forward
pub struct Renderer {
    focal_length: f32,
    interpolation_mode: InterpMode,
    cull_mode: CullMode,
    render_mode: RenderMode,
    depth_test: DepthTest,
    zbuffer: Vec<f32>,
    zbuffer_res: (usize, usize),
}
impl Renderer {
    /// Creates a new Renderer. Z-buffer memory is allocated to match the given screen dimensions.
    pub fn new(
        screen: &Screen, interpolation_mode: InterpMode, cull_mode: CullMode,
        render_mode: RenderMode, depth_test: DepthTest
    ) -> Self {
        let res = (screen.width_pix as usize, screen.height_pix as usize);
        let zbuffer = vec![DepthTest::DONT_CARE; (res.0 * res.1)];

        Renderer {
            focal_length: 1.0,
            interpolation_mode,
            cull_mode,
            render_mode,
            depth_test,
            zbuffer,
            zbuffer_res: res,
        }
    }

    /// Clears the z-buffer by filling it with the starting value defined by the depth test.
    /// If the screen resolution has changed since the last render, it reallocates the z-buffer to match the new dimensions.
    fn clear_zbuffer(&mut self, screen: &Screen) {
        let width = screen.width_pix as usize;
        let height = screen.height_pix as usize;
        let value = self.depth_test.starting_value();
        if self.zbuffer_res.0 != width || self.zbuffer_res.1 != height {
            self.zbuffer_res = (width, height);
            self.zbuffer = vec![value; width * height];
        }
        else {
            self.zbuffer.fill(value);
        }

    }

    /// Renders the scene from the perspective of the given camera onto `self.screen`.
    pub fn render_scene_from_camera(&mut self, camera: &Camera, screen: &mut Screen) {
        self.clear_zbuffer(screen);
        let view = Transform::inverse(&camera.transform);
        self.focal_length = camera.get_focal_length(screen.get_width_units());
        for object in &camera.scene.objects {
            self.render_object(object, &view, screen);
        }
    }

    /// Renders a single object by applying the view transformation and then rasterizing its primitives.
    fn render_object(&mut self, object: &Object, view: &Transform, screen: &mut Screen) {
        let combined_transform = view.combine_with(&object.transform);
        let mut ctx = RenderingContext {
            screen,
            transform: &combined_transform,
            shader: &|v| v.color,   // TODO support proper shaders
        };
        for primitive in &object.mesh.primitives {
            match primitive {
                Primitive::Triangle(t) => self.render_triangle(t, &mut ctx),
                Primitive::Line(v1, v2) => self.render_line(v1, v2, &mut ctx),
                Primitive::Point(v) => self.render_point(v, &mut ctx),
            }
        }
    }

    /// Renders a triangle by drawing the color of every pixel it covers onto the screen.
    /// It starts by applying the combined view and object transformations to the triangle vertices,
    /// then projects them to 2D screen space, and finally converts those to pixel coordinates.
    /// Every pixel in the triangle should be colored according to their interpolated attributes.
    fn render_triangle(&mut self, triangle: &Triangle, ctx: &mut RenderingContext) {
        // Apply transformations to the triangle vertices
        let a = ctx.transform.apply_to(&triangle.a.pos);
        let b = ctx.transform.apply_to(&triangle.b.pos);
        let c = ctx.transform.apply_to(&triangle.c.pos);

        let cam_space_tri = Triangle::new(
            Vertex::new(a, triangle.a.color),
            Vertex::new(b, triangle.b.color),
            Vertex::new(c, triangle.c.color),
        );

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
        let pa = ctx.screen.world_to_screen_coords(sa);
        let pb = ctx.screen.world_to_screen_coords(sb);
        let pc = ctx.screen.world_to_screen_coords(sc);
        if !ctx.screen.in_bounds(pa.x, pa.y) || !ctx.screen.in_bounds(pb.x, pb.y)
            || !ctx.screen.in_bounds(pc.x, pc.y) {
            return; // TODO lacking clipping so just discard the triangle
        }

        match self.render_mode {
            RenderMode::Solid => self.rasterize_triangle(pa, pb, pc, &cam_space_tri, ctx),
            RenderMode::Wireframe => {
                self.rasterize_line(pa, pb, &cam_space_tri.a, &cam_space_tri.b, ctx);
                self.rasterize_line(pb, pc, &cam_space_tri.b, &cam_space_tri.c, ctx);
                self.rasterize_line(pc, pa, &cam_space_tri.c, &cam_space_tri.a, ctx);
            }
        }
    }

    /// Renders a line by drawing the color of every pixel it covers onto the screen.
    /// It starts by applying the combined view and object transformations to the line endpoints,
    /// then projects them to 2D screen space, and finally converts those to pixel coordinates.
    /// Every pixel in the line should be colored according to their interpolated attributes.
    fn render_line(&mut self, v1: &Vertex, v2: &Vertex, ctx: &mut RenderingContext) {
        let a = ctx.transform.apply_to(&v1.pos);
        let b = ctx.transform.apply_to(&v2.pos);

        let cam_v1 = Vertex::new(a, v1.color);
        let cam_v2 = Vertex::new(b, v2.color);

        let sa = self.perspective_project(&a);
        let sb = self.perspective_project(&b);

        let pa = ctx.screen.world_to_screen_coords(sa);
        let pb = ctx.screen.world_to_screen_coords(sb);

        if ctx.screen.in_bounds(pa.x, pa.y) && ctx.screen.in_bounds(pb.x, pb.y) {
            self.rasterize_line(pa, pb, &cam_v1, &cam_v2, ctx);
        }
    }

    /// Renders a point by drawing its color in its calculated screen position.
    /// This calculation starts by applying the combined view and object transformations to it,
    /// then projects it to 2D screen space, and finally converts that to pixel coordinates.
    fn render_point(&mut self, vertex: &Vertex, ctx: &mut RenderingContext) {
        let v = ctx.transform.apply_to(&vertex.pos);
        let cam_v = Vertex::new(v, vertex.color);
        let sv = self.perspective_project(&v);
        let pv = ctx.screen.world_to_screen_coords(sv);
        if ctx.screen.in_bounds(pv.x, pv.y) {
            self.draw_pixel(pv, &cam_v, ctx);
        }
    }

    /// Rasterizes a triangle by filling in the color of every pixel it covers onto the screen.
    /// It precomputes triangle constants to make the inner loop faster, avoiding the overhead of
    /// barycentric_coords function.
    fn rasterize_triangle(&mut self, pa: Pixel, pb: Pixel, pc: Pixel, triangle: &Triangle, ctx: &mut RenderingContext) {
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
                let v2 = Vec2::new((x - pa.x) as f32 + 0.5, (y - pa.y) as f32 + 0.5);
                let d20 = v2.dot(&v0);
                let d21 = v2.dot(&v1);
                let v = (d11 * d20 - d01 * d21) * inv_denom;
                let w = (d00 * d21 - d01 * d20) * inv_denom;
                let u = 1.0 - v - w;
                if u >= -FP_TOLERANCE && v >= -FP_TOLERANCE && w >= -FP_TOLERANCE {
                    let pixel_attributes = self.lerp_pixel_in_triangle(u, v, &triangle);
                    self.draw_pixel(Pixel::new(x, y), &pixel_attributes, ctx);
                }
            }
        }
    }

    /// Rasterizes a line by filling in the color of every pixel it covers onto the screen.
    fn rasterize_line(&mut self, p1: Pixel, p2: Pixel, v1: &Vertex, v2: &Vertex, ctx: &mut RenderingContext) {
        let dx = p2.x - p1.x;
        let dy = p2.y - p1.y;
        if dx == 0 && dy == 0 {
            self.draw_pixel(p1, v1, ctx);
            return;
        }

        if dy == 0 {    // screen can already handle horizontal spans, so we can optimize for that case
            self.draw_hline(p1.y, p1.x, p2.x, &v1, &v2, ctx);
            return;
        }
        // vertical spans are not supported by the screen, so we just use the general case

        let steps = dx.abs().max(dy.abs());
        let slope = Vec2::new(dx as f32 / steps as f32, dy as f32 / steps as f32);
        for i in 0..=steps {
            let t = i as f32 / steps as f32;
            let p = p1.add(&Pixel::from_vec2(&slope.scale(t)));
            let pixel_attributes = self.lerp_pixel_in_line(t, v1, v2);
            self.draw_pixel(p, &pixel_attributes, ctx);
        }
    }

    /// Draws a pixel on the screen if it passes the depth test.
    /// The color of the pixel is determined by the shader function in the rendering context, which takes the vertex attributes as input.
    /// This function assumes the pixel coordinates are valid and does not perform any bounds checking.
    fn draw_pixel(&mut self, p: Pixel, v: &Vertex, ctx: &mut RenderingContext) {
        let idx = (self.zbuffer_res.0 as i32) * (p.y) + (p.x);
        self.test_and_draw(idx, v, ctx);
    }

    /// Draws a horizontal line on the screen from (x_start, y) to (x_end, y).
    /// It is optimized to avoid the overhead of index calculation.
    fn draw_hline(&mut self, y: i32, x_start: i32, x_end: i32, v1: &Vertex, v2: &Vertex, ctx: &mut RenderingContext) {
        let dx = x_end - x_start;
        let idx = (self.zbuffer_res.0 as i32) * y + (x_start);

        for x in 0..=dx {
            let t = x as f32 / dx as f32;
            let pixel_attributes = self.lerp_pixel_in_line(t, v1, v2);
            self.test_and_draw(idx + x, &pixel_attributes, ctx);
        }
    }

    /// Tests the depth of the pixel against the z-buffer and draws it if it passes.
    /// This is used by both draw_pixel and draw_hline to avoid code duplication.
    fn test_and_draw(&mut self, idx: i32, v: &Vertex, ctx: &mut RenderingContext) {
        if self.depth_test.test(v.pos.z, self.zbuffer[idx as usize]) {
            self.zbuffer[idx as usize] = v.pos.z;
            let color = (ctx.shader)(v);    // we delayed this call to save computation
            ctx.screen.fast_draw_pixel(idx, &color);
        }
    }

    /// Returns the interpolated vertex attributes at a pixel inside a triangle,
    /// given its barycentric coordinates `alpha` and `beta`.
    fn lerp_pixel_in_triangle(&self, alpha: f32, beta: f32, triangle: &Triangle) -> Vertex {
        let pixel_attributes = match self.interpolation_mode {
            InterpMode::Linear => triangle.tri_lerp(alpha, beta),
            InterpMode::PerspectiveCorrect => triangle.ztri_lerp(alpha, beta)
        };
        pixel_attributes
    }

    /// Returns the interpolated vertex attributes at a pixel inside a line,
    /// given its interpolation factor `t` (0.0 at the start vertex and 1.0 at the end vertex).
    fn lerp_pixel_in_line(&self, t: f32, v1: &Vertex, v2: &Vertex) -> Vertex {
        let pixel_attributes = match self.interpolation_mode {
            InterpMode::Linear => Vertex::lerp(v1, v2, t),
            InterpMode::PerspectiveCorrect => Vertex::zlerp(v1, v2, t)
        };
        pixel_attributes
    }

    /// Projects a 3D point in camera space to 2D screen space using perspective projection.
    fn perspective_project(&self, point: &Vec3) -> Vec2 {
        Vec2::new(point.x, point.y).scale(self.focal_length/point.z)
    }
}