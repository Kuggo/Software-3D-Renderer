use crate::{Camera, Screen};
use crate::geometry::*;
use crate::utils::{Color, Vec2, Vec3};

/// The Renderer is responsible for taking a Scene and a Camera, and producing a 2D image on the Screen.
/// Right-handed
/// X+ → right
/// Y+ → up
/// Z+ → forward
pub struct Renderer {
    focal_length: f32,
}
impl Renderer {
    pub fn new() -> Self {
        Renderer { focal_length: 1.0 }
    }

    pub fn render_scene_from_camera(&mut self, camera: &mut Camera) {
        let view = Transform::inverse(&camera.transform);
        self.focal_length = camera.get_focal_length();
        for object in &camera.scene.objects {
            self.render_object(object, &mut camera.screen, &view);
        }
    }

    pub fn render_object(&mut self, object: &Object, screen: &mut Screen, view: &Transform) {
        for primitive in &object.mesh.primitives {
            let combined_transform = view.combine_with(&object.transform);
            match primitive {
                Primitive::Triangle(t) => self.rasterize_triangle(t, &combined_transform, screen),
                //Primitive::Line(l) => self.draw_line(l, &object.transform, camera),
                //Primitive::Point(p) => self.draw_point(p, &object.transform, camera),
            }
        }
    }

    fn rasterize_triangle(&mut self, triangle: &Triangle, transform: &Transform, screen: &mut Screen) {
        // Apply transformations to the triangle vertices
        let a = transform.apply_to(&triangle.a.pos);
        let b = transform.apply_to(&triangle.b.pos);
        let c = transform.apply_to(&triangle.c.pos);

        // Project the vertices to 2D screen space
        let a = self.perspective_project(&a);
        let b = self.perspective_project(&b);
        let c = self.perspective_project(&c);

        // convert from screen space to pixel coordinates
        let (ax, ay) = screen.world_to_screen_coords(a);
        let (bx, by) = screen.world_to_screen_coords(b);
        let (cx, cy) = screen.world_to_screen_coords(c);

        // For now, we'll just draw the vertices
        screen.draw_pixel(ax, ay, Color::RED.to_argb());
        screen.draw_pixel(bx, by, Color::GREEN.to_argb());
        screen.draw_pixel(cx, cy, Color::BLUE.to_argb());

        // TODO Rasterize the triangle and fill pixels on the screen
    }

    fn perspective_project(&self, point: &Vec3) -> Vec2 {
        Vec2::new(point.x, point.y).scale(self.focal_length/point.z)
    }
}