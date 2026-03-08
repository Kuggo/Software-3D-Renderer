use crate::{Camera, Screen};
use crate::geometry::*;
use crate::utils::{Color, Vec2, Vec3};

/// The Renderer is responsible for taking a Scene and a Camera, and producing a 2D image on the Screen.
/// Right-handed
/// X+ → right
/// Y+ → up
/// Z+ → forward
pub struct Renderer<'a> {
    focal_length: f32,
    pub screen: &'a mut Screen,
}
impl<'a> Renderer<'a> {
    /// Creates a new Renderer with a reference to the Screen it will draw on.
    pub fn new_to_screen(screen: &'a mut Screen) -> Self {
        Renderer { focal_length: 1.0, screen }
    }

    /// Renders the scene from the perspective of the given camera onto `self.screen`.
    pub fn render_scene_from_camera(&mut self, camera: &mut Camera) {
        let view = Transform::inverse(&camera.transform);
        self.focal_length = camera.get_focal_length( self.screen.get_width_units() );
        for object in &camera.scene.objects {
            self.render_object(object, &view);
        }
    }

    /// Renders a single object by applying the view transformation and then rasterizing its primitives.
    fn render_object(&mut self, object: &Object, view: &Transform) {
        let combined_transform = view.combine_with(&object.transform);
        for primitive in &object.mesh.primitives {
            match primitive {
                Primitive::Triangle(t) => self.rasterize_triangle(t, &combined_transform),
                //Primitive::Line(l) => self.draw_line(l, &object.transform, camera),
                //Primitive::Point(p) => self.draw_point(p, &object.transform, camera),
            }
        }
    }

    /// Rasterizes a triangle by drawing the color of every pixel it covers onto the screen.
    /// It starts by applying the combined view and object transformations to the triangle vertices,
    /// then projects them to 2D screen space, and finally converts those to pixel coordinates.
    /// Every pixel covered by the triangle should be colored in, but for now we just draw the vertices as a proof of concept.
    fn rasterize_triangle(&mut self, triangle: &Triangle, transform: &Transform) {
        // Apply transformations to the triangle vertices
        let a = transform.apply_to(&triangle.a.pos);
        let b = transform.apply_to(&triangle.b.pos);
        let c = transform.apply_to(&triangle.c.pos);

        // Project the vertices to 2D screen space
        let a = self.perspective_project(&a);
        let b = self.perspective_project(&b);
        let c = self.perspective_project(&c);

        // convert from screen space to pixel coordinates
        let (ax, ay) = self.screen.world_to_screen_coords(a);
        let (bx, by) = self.screen.world_to_screen_coords(b);
        let (cx, cy) = self.screen.world_to_screen_coords(c);

        // For now, we'll just draw the vertices
        self.screen.draw_pixel(ax, ay, Color::RED);
        self.screen.draw_pixel(bx, by, Color::GREEN);
        self.screen.draw_pixel(cx, cy, Color::BLUE);

        // TODO Rasterize the triangle and fill pixels on the screen
    }

    /// Projects a 3D point in camera space to 2D screen space using perspective projection.
    fn perspective_project(&self, point: &Vec3) -> Vec2 {
        Vec2::new(point.x, point.y).scale(self.focal_length/point.z)
    }
}