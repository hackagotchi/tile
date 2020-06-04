#[derive(Debug)]
pub struct Camera {
    pub eye: nalgebra::Point3<f32>,
    pub target: nalgebra::Point3<f32>,
    up: nalgebra::Vector3<f32>,
    aspect: f32,
    pub fovy: f32,
    znear: f32,
    zfar: f32,
}
impl Camera {
    pub const DEFAULT_ANGLE: f32 = std::f32::consts::PI / 2.0;
    pub const DEFAULT_DISTANCE: f32 = 6.0;

    pub fn new(w: f32, h: f32) -> Self {
        let mut c = Camera {
            eye: nalgebra::Point3::new(0.0, 0.0, 3.0),
            target: nalgebra::Point3::new(0.0, 0.0, 0.0),
            up: nalgebra::Vector3::z(),
            aspect: 0.0,
            fovy: std::f32::consts::PI / 2.0,
            znear: 0.1,
            zfar: 100.0,
        };
        c.set_angle(Self::DEFAULT_ANGLE, Self::DEFAULT_DISTANCE);
        c.resize(w, h);
        c
    }

    pub fn resize(&mut self, w: f32, h: f32) {
        self.aspect = w / h;
    }

    pub fn set_angle(&mut self, angle: f32, distance: f32) {
        let (sin, cos) = angle.sin_cos();
        self.eye.x = cos * distance;
        self.eye.y = sin * distance;
    }

    pub fn build_view_projection_matrix(&self) -> nalgebra::Matrix4<f32> {
        let view = nalgebra::Matrix4::look_at_rh(
            &(self.eye.coords + self.target.coords).into(),
            &self.target,
            &self.up
        );
        let proj =
            nalgebra::Matrix4::new_perspective(self.aspect, self.fovy, self.znear, self.zfar);

        return proj * view;
    }
}
