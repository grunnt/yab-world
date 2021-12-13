#![allow(dead_code)]
use glm::{Mat4, Vec2, Vec3};

pub struct PerspectiveCamera {
    pub position: Vec3,
    front: Vec3,
    right: Vec3,
    up: Vec3,
    world_up: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pitch_limit: f32,
    projection: Mat4,
    view: Mat4,
    aspect: f32,
    fovy: f32,
    fovx: f32,
    znear: f32,
    zfar: f32,
}

impl PerspectiveCamera {
    pub fn new(
        position: Vec3,
        yaw: f32,
        pitch: f32,
        aspect: f32,
        fovy: f32,
        znear: f32,
        zfar: f32,
    ) -> PerspectiveCamera {
        let world_up = Vec3::z();
        let (front, right, up) = calculate_camera_vectors(yaw, pitch, &world_up);
        let mut camera = PerspectiveCamera {
            position,
            front,
            right,
            up,
            world_up,
            yaw: 0.0,
            pitch: 0.0,
            pitch_limit: std::f32::consts::FRAC_PI_2 * 0.9,
            projection: Mat4::identity(),
            view: Mat4::identity(),
            aspect,
            fovy,
            fovx: calculate_fovx(fovy, aspect),
            znear,
            zfar,
        };
        camera.update();
        camera
    }

    /// Get the direction in which the camera is facing
    pub fn get_direction(&self) -> &Vec3 {
        &self.front
    }

    /// Get roll, pitch and yaw values
    pub fn get_angles(&self) -> (f32, f32) {
        (self.pitch, self.yaw)
    }

    pub fn frustum_checker(&self) -> FrustumChecker {
        FrustumChecker {
            position: self.position,
            front: self.front,
            fovx: self.fovx,
        }
    }

    /// Get a reference to the inner projection matrix of this camera
    pub fn get_projection(&self) -> &Mat4 {
        &self.projection
    }

    /// Get view matrix of this camera
    pub fn get_view(&self) -> &Mat4 {
        &self.view
    }

    pub fn update(&mut self) {
        self.view = glm::look_at(&self.position, &(self.position + self.front), &self.up);
        self.projection = glm::perspective(self.aspect, self.fovy, self.znear, self.zfar);
        self.fovx = calculate_fovx(self.fovy, self.aspect);
    }

    /// Rotate camera using relative mouse movement over screen pixels.
    pub fn rotate(&mut self, delta_yaw: f32, delta_pitch: f32) {
        self.yaw += delta_yaw;
        self.pitch += delta_pitch;
        if self.pitch > self.pitch_limit {
            self.pitch = self.pitch_limit;
        } else if self.pitch < -self.pitch_limit {
            self.pitch = -self.pitch_limit;
        }
        let (front, right, up) = calculate_camera_vectors(self.yaw, self.pitch, &self.world_up);
        self.front = front;
        self.right = right;
        self.up = up;
        self.update();
    }

    /// Move the camera in camera space
    pub fn translate(&mut self, direction: CameraMovement, distance: f32) {
        self.position += match direction {
            CameraMovement::Forward => self.front,
            CameraMovement::Back => -self.front,
            CameraMovement::Right => self.right,
            CameraMovement::Left => -self.right,
            CameraMovement::Up => self.up,
            CameraMovement::Down => -self.up,
        } * distance;
        self.update();
    }

    pub fn set_aspect(&mut self, aspect: f32) {
        self.aspect = aspect;
        self.update();
    }
}

#[derive(Copy, Clone, Debug)]
pub enum CameraMovement {
    Left,
    Right,
    Forward,
    Back,
    Up,
    Down,
}

// calculates the front vector from the Camera's (updated) Euler Angles
fn calculate_camera_vectors(yaw: f32, pitch: f32, world_up: &Vec3) -> (Vec3, Vec3, Vec3) {
    // calculate the new Front vector
    let front = Vec3::new(
        yaw.cos() * pitch.cos(),
        yaw.sin() * pitch.cos(),
        pitch.sin(),
    );
    let front = front.normalize();
    // also re-calculate the Right and Up vector
    let right = glm::normalize(&glm::cross(&front, world_up));
    let up = glm::normalize(&glm::cross(&right, &front));
    (front, right, up)
}

pub struct OrthographicCamera {
    pub position: Vec2,
    projection: Mat4,
}

impl OrthographicCamera {
    pub fn new(width: u32, height: u32) -> OrthographicCamera {
        OrthographicCamera {
            position: Vec2::new(0.0, 0.0),
            projection: orthographic(width as f32, height as f32),
        }
    }

    pub fn set_size(&mut self, width: u32, height: u32) {
        self.projection = orthographic(width as f32, height as f32);
    }

    /// Move the camera
    pub fn translate(&mut self, translation: Vec2) {
        self.position = self.position + translation;
    }

    /// Get a reference to the inner projection matrix of this camera
    pub fn get_projection(&self) -> &Mat4 {
        &self.projection
    }

    /// Get view matrix of this camera
    // pub fn get_view(&self) -> Mat4 {
    //     Mat4::from_translation()
    // }

    /// Get combined view * projection matrix
    pub fn get_projection_view(&self) -> Mat4 {
        self.projection //  * self.get_view()
    }
}

fn orthographic(width: f32, height: f32) -> Mat4 {
    Mat4::new_orthographic(0.0, width, height, 0.0, -1.0, 1.0)
}

fn calculate_fovx(fovy: f32, aspect: f32) -> f32 {
    2.0 * ((fovy * 0.5).tan() * aspect).atan()
}

pub struct FrustumChecker {
    position: Vec3,
    front: Vec3,
    fovx: f32,
}

impl FrustumChecker {
    pub fn is_inside_frustrum_xy(&self, p: Vec3) -> bool {
        let p_vec = p - self.position;
        let p_angle = glm::angle(&self.front.xy(), &p_vec.xy());
        p_angle < self.fovx * 0.6
    }
}
