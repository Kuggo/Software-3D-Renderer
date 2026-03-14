use crate::{Camera, Screen};
use crate::geometry::*;
use crate::shader::{BaseShader};
use crate::utils::{fp_equals, Pixel, Vec2, Vec3, FP_TOLERANCE};

#[derive(Debug, Copy, Clone)]
pub enum InterpMode {
    Linear,
    DepthCorrect,
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


struct FaceContext<'a> {
    pixels: &'a [Pixel],
    verts: &'a [u32],
    depths: &'a [f32],
}


struct RenderingContext<'a> {
    screen: &'a mut Screen,
    transform: &'a Transform,
    shader: &'a dyn BaseShader,
    mesh: &'a Mesh,
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
        if !object.material.shader.validate_mesh(&object.mesh) {
            println!("Invalid mesh for shader, skipping object");
            return;
        }

        let combined_transform = view.combine_with(&object.transform);
        let mut ctx = RenderingContext {
            screen,
            transform: &combined_transform,
            shader: object.material.shader,
            mesh: &object.mesh,
        };
        for primitive in &object.mesh.primitives {
            match primitive {
                Primitive::Triangle(v1, v2, v3) =>
                    self.render_triangle(&[*v1, *v2, *v3], &mut ctx),
                Primitive::Line(v1, v2) =>
                    self.render_line(&[*v1, *v2], &mut ctx),
                Primitive::Point(v) =>
                    self.render_point(*v, &mut ctx),
            }
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////////////////
    // Primitive rendering
    ////////////////////////////////////////////////////////////////////////////////////////////////

    /// Renders a triangle by drawing the color of every pixel it covers onto the screen.
    /// It starts by applying the combined view and object transformations to the triangle vertices,
    /// then projects them to 2D screen space, and finally converts those to pixel coordinates.
    /// Every pixel in the triangle should be colored according to their interpolated attributes.
    fn render_triangle(&mut self, tri: &[u32; 3], ctx: &mut RenderingContext) {
        let v1_pos = &ctx.mesh.positions[tri[0] as usize];
        let v2_pos = &ctx.mesh.positions[tri[1] as usize];
        let v3_pos = &ctx.mesh.positions[tri[2] as usize];

        // Apply transformations to the triangle vertices
        let a = ctx.transform.apply_to(v1_pos);
        let b = ctx.transform.apply_to(v2_pos);
        let c = ctx.transform.apply_to(v3_pos);

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

        let mut face = FaceContext {
            pixels: &[pa, pb, pc],
            verts: tri,
            depths: &[a.z, b.z, c.z],
        };
        match self.render_mode {
            RenderMode::Solid => {
                self.scanline_triangle(&face, ctx)
            },
            RenderMode::Wireframe => {
                self.rasterize_line(&face, ctx);
                face.pixels = &face.pixels[1..];
                face.verts = &tri[1..];
                face.depths = &face.depths[1..];
                self.rasterize_line(&face, ctx);
                let face = FaceContext {
                    pixels: &[pc, pa],
                    verts: &[tri[2], tri[0]],
                    depths: &[c.z, a.z],
                };
                self.rasterize_line(&face, ctx);
            }
        }
    }

    /// Renders a line by drawing the color of every pixel it covers onto the screen.
    /// It starts by applying the combined view and object transformations to the line endpoints,
    /// then projects them to 2D screen space, and finally converts those to pixel coordinates.
    /// Every pixel in the line should be colored according to their interpolated attributes.
    fn render_line(&mut self, line: &[u32], ctx: &mut RenderingContext) {
        let v1_pos = &ctx.mesh.positions[line[0] as usize];
        let v2_pos = &ctx.mesh.positions[line[1] as usize];

        let cam_v1 = ctx.transform.apply_to(v1_pos);
        let cam_v2 = ctx.transform.apply_to(v2_pos);

        let sa = self.perspective_project(&cam_v1);
        let sb = self.perspective_project(&cam_v2);

        let pa = ctx.screen.world_to_screen_coords(sa);
        let pb = ctx.screen.world_to_screen_coords(sb);

        if ctx.screen.in_bounds(pa.x, pa.y) && ctx.screen.in_bounds(pb.x, pb.y) {
            let face = FaceContext {
                pixels: &[pa, pb],
                verts: line,
                depths: &[cam_v1.z, cam_v2.z],
            };
            self.rasterize_line(&face, ctx);
        }
    }

    /// Renders a point by drawing its color in its calculated screen position.
    /// This calculation starts by applying the combined view and object transformations to it,
    /// then projects it to 2D screen space, and finally converts that to pixel coordinates.
    fn render_point(&mut self, v1: u32, ctx: &mut RenderingContext) {
        let v_pos = ctx.mesh.positions[v1 as usize];
        let cam_v = ctx.transform.apply_to(&v_pos);
        let sv = self.perspective_project(&cam_v);
        let pv = ctx.screen.world_to_screen_coords(sv);
        if ctx.screen.in_bounds(pv.x, pv.y) {
            let face = FaceContext {
                pixels: &[pv],
                verts: &[v1],
                depths: &[cam_v.z],
            };
            self.draw_pixel(pv, &[1.0], &face, ctx);
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////////////////
    // Triangle rasterization
    ////////////////////////////////////////////////////////////////////////////////////////////////

    /// Rasterizes a triangle by filling in the color of every pixel it covers onto the screen.
    /// It precomputes triangle constants to make the inner loop faster
    fn rasterize_triangle(&mut self, face_ctx: &FaceContext, ctx: &mut RenderingContext) {
        let (pa, pb, pc) = (face_ctx.pixels[0], face_ctx.pixels[1], face_ctx.pixels[2]);
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
                    self.draw_pixel(Pixel::new(x, y), &[u, v, w], face_ctx, ctx);
                }
            }
        }
    }

    /// Rasterizes a triangle by filling in the color of every pixel it covers onto the screen.
    /// It uses the scanline algorithm, which is more efficient than barycentric coordinate method
    /// because it avoids the overhead of the dot products and multiplications in the inner loop.
    fn scanline_triangle(&mut self, face_ctx: &FaceContext, ctx: &mut RenderingContext) {
        // Walk along smaller edge and big edge and draw horizontal lines.
        // switch to other small edge when we reach the end of current one.
        let mut verts = [0usize, 1, 2];
        verts.sort_by_key(|&i| {face_ctx.pixels[i].y});
        // long edge is verts[0] to verts[2]
        // small edges are verts[0] to verts[1] and verts[1] to verts[2]
        let (p0, p1, p2) = (face_ctx.pixels[verts[0]], face_ctx.pixels[verts[1]], face_ctx.pixels[verts[2]]);
        let uvs = [Vec2::X_AXIS, Vec2::Y_AXIS, Vec2::ZERO];
        let (uv0, uv1, uv2) = (uvs[verts[0]], uvs[verts[1]], uvs[verts[2]]);

        let long_slope_x      = (p2.x - p0.x) as f32 / (p2.y - p0.y) as f32;
        let long_slope_uv      = (uv2 - uv0) * (1.0 / (p2.y - p0.y) as f32);
        let mut short_slope_x = (p1.x - p0.x) as f32 / (p1.y - p0.y) as f32;
        let mut short_slope_uv = (uv1 - uv0) * (1.0 / (p1.y - p0.y) as f32);

        let mut long_x  = p0.x as f32 + 0.5;
        let mut long_uv  = uv0;
        let mut short_x = p0.x as f32 + 0.5;
        let mut short_uv = uv0;
        for y in (p0.y)..p1.y {
            self.draw_hline(y, short_x as i32, long_x as i32, short_uv, long_uv, face_ctx, ctx);
            long_x += long_slope_x;
            long_uv.x += long_slope_uv.x;
            long_uv.y += long_slope_uv.y;
            short_x += short_slope_x;
            short_uv.x += short_slope_uv.x;
            short_uv.y += short_slope_uv.y;
        }

        short_slope_x = (p2.x - p1.x) as f32 / (p2.y - p1.y) as f32;
        short_slope_uv  = (uv2 - uv1) * (1.0 / (p2.y - p1.y) as f32);

        short_x = p1.x as f32 + 0.5;
        short_uv = uv1;
        for y in p1.y..p2.y {
            self.draw_hline(y, short_x as i32, long_x as i32, short_uv, long_uv, face_ctx, ctx);
            long_x += long_slope_x;
            long_uv.x += long_slope_uv.x;
            long_uv.y += long_slope_uv.y;
            short_x += short_slope_x;
            short_uv.x += short_slope_uv.x;
            short_uv.y += short_slope_uv.y;
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////////////////
    // Line drawing
    ////////////////////////////////////////////////////////////////////////////////////////////////

    /// Rasterizes a line by filling in the color of every pixel it covers onto the screen.
    fn rasterize_line(&mut self, face_ctx: &FaceContext, ctx: &mut RenderingContext) {
        let (p1, p2) = (face_ctx.pixels[0], face_ctx.pixels[1]);
        let dx = p2.x - p1.x;
        let dy = p2.y - p1.y;
        if dx == 0 && dy == 0 {
            self.draw_pixel(p1, &[1.0, 0.0], face_ctx, ctx);
            return;
        }

        if dy == 0 {    // screen can already handle horizontal spans, so we can optimize for that case
            let (b1, b2) = (Vec2::new(1.0, 0.0), Vec2::new(0.0, 1.0));
            self.draw_hline(p1.y, p1.x, p2.x, b1, b2, face_ctx, ctx);
            return;
        }

        let steps = dx.abs().max(dy.abs());
        let slope = Vec2::new(dx as f32 / steps as f32, dy as f32 / steps as f32);
        for i in 0..=steps {
            let t = i as f32 / steps as f32;
            let p = p1.add(&Pixel::from_vec2(&(slope * t)));
            let bary = &[1.0 - t, t];
            self.draw_pixel(p, bary, face_ctx, ctx);
        }
    }

    /// Draws a horizontal line on the screen from (x_start, y) to (x_end, y).
    /// It is optimized to avoid the overhead of index calculation.
    fn draw_hline(
        &mut self, y: i32, mut x_start: i32, mut x_end: i32, mut start_bary: Vec2, mut end_bary: Vec2,
        face_ctx: &FaceContext, ctx: &mut RenderingContext
    ) {
        if x_end < x_start {
            (x_start, x_end) = (x_end, x_start);
            (start_bary, end_bary) = (end_bary, start_bary);
        }
        let dx = x_end - x_start;

        let mut uv = start_bary;
        let duv = (end_bary - start_bary) * (1.0 / dx as f32);

        let idx = ((self.zbuffer_res.0 as i32) * y + x_start) as usize;
        for x in 0..=dx as usize {
            self.test_and_draw(idx + x, face_ctx, &[uv.x, uv.y, 1.0-uv.x-uv.y], ctx);
            uv.x += duv.x;
            uv.y += duv.y;
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////////////////
    // Pixel drawing and depth testing
    ////////////////////////////////////////////////////////////////////////////////////////////////

    /// Draws a pixel on the screen if it passes the depth test.
    /// The color of the pixel is determined by the shader function in the rendering context, which takes the vertex attributes as input.
    /// This function assumes the pixel coordinates are valid and does not perform any bounds checking.
    fn draw_pixel(&mut self, p: Pixel, bary: &[f32], face_ctx: &FaceContext, ctx: &mut RenderingContext) {
        let idx = (self.zbuffer_res.0 as i32) * (p.y) + (p.x);
        self.test_and_draw(idx as usize, face_ctx, bary, ctx);
    }

    /// Tests the depth of the pixel against the z-buffer and draws it if it passes.
    /// This is used by both draw_pixel and draw_hline to avoid code duplication.
    fn test_and_draw(&mut self, idx: usize, face_ctx: &FaceContext, bary: &[f32], ctx: &mut RenderingContext) {
        let z = self.interpolate_depth(face_ctx, bary);
        if self.depth_test.test(z, self.zbuffer[idx]) {
            self.zbuffer[idx] = z;
            let weights = &mut [0f32; 3];   // so far 3 is the max
            self.adjust_bary_weights(bary, face_ctx.depths, z, weights);
            let color = ctx.shader.shade(ctx.mesh, face_ctx.verts, weights);    // we delayed this call to save computation
            ctx.screen.fast_draw_pixel(idx, &color);
        }
    }

    ////////////////////////////////////////////////////////////////////////////////////////////////
    // Perspective correction and interpolation
    ////////////////////////////////////////////////////////////////////////////////////////////////

    /// Interpolates the depth of a pixel based on the interpolation mode.
    fn interpolate_depth(&self, face_ctx: &FaceContext, bary: &[f32]) -> f32 {
        let len = bary.len();
        let z = match self.interpolation_mode {
            InterpMode::Linear => {
                let mut z = 0.0;
                for i in 0..len {
                    z += face_ctx.depths[i] * bary[i];
                }
                z
            },
            InterpMode::DepthCorrect => {
                let mut inv_z = 0.0;
                for i in 0..len {
                    inv_z += (1.0/face_ctx.depths[i]) * bary[i];
                }
                1.0 / inv_z
            },
        };
        z
    }

    /// Adjusts the barycentric weights according to the interpolation mode.
    fn adjust_bary_weights(&self, bary: &[f32], depths: &[f32], z: f32, weights: &mut [f32]) {
        match self.interpolation_mode {
            InterpMode::Linear => {
                for i in 0..depths.len() {
                    weights[i] = bary[i];
                }
            }
            InterpMode::DepthCorrect => {
                for i in 0..depths.len() {
                    weights[i] = (bary[i] / depths[i]) * z;
                }
            }
        }
    }

    /// Projects a 3D point in camera space to 2D screen space using perspective projection.
    fn perspective_project(&self, point: &Vec3) -> Vec2 {
        Vec2::new(point.x, point.y) * (self.focal_length / point.z)
    }
}

