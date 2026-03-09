use crate::{Camera, Screen};
use crate::geometry::*;
use crate::utils::{Color, Pixel, Vec2, Vec3};

#[derive(Debug, Copy, Clone)]
pub enum InterpMode {
    Linear,
    PerspectiveCorrect,
}

/// The Renderer is responsible for taking a Scene and a Camera, and producing a 2D image on the Screen.
/// Right-handed
/// X+ -> right
/// Y+ -> up
/// Z+ -> forward
pub struct Renderer<'a> {
    focal_length: f32,
    pub screen: &'a mut Screen,
    mode: InterpMode,
}
impl<'a> Renderer<'a> {
    /// Creates a new Renderer with a reference to the Screen it will draw on.
    pub fn new_to_screen(screen: &'a mut Screen, mode: InterpMode) -> Self {
        Renderer { focal_length: 1.0, screen, mode }
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

        // convert from screen space to pixel coordinates
        let pa = self.screen.world_to_screen_coords(sa);
        let pb = self.screen.world_to_screen_coords(sb);
        let pc = self.screen.world_to_screen_coords(sc);

        // Draw the edges of the triangle as a proof of concept
        self.rasterize_line(pa, pb, &triangle.a, &triangle.b);
        self.rasterize_line(pb, pc, &triangle.b, &triangle.c);
        self.rasterize_line(pc, pa, &triangle.c, &triangle.a);
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
    fn rasterize_triangle(&mut self, triangle: &Triangle, transform: &Transform) {
        todo!()
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
            match self.mode {
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