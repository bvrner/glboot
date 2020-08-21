use cgmath::{InnerSpace, Point2, Quaternion, Vector3};

#[derive(Debug, Copy, Clone)]
pub struct ArcBall {
    // the dimensions of the window
    win_size: (f32, f32),
    // the point the rotation currently is
    current: Point2<f32>,
    // the vector to the center of the ball when the click occurred
    click_vec: Vector3<f32>,
    // the vector to the center of the ball when the drag occurred
    drag_vec: Vector3<f32>,
    // current rotation
    this_rot: Quaternion<f32>,
    // last rotation
    last_rot: Quaternion<f32>,
    // is a drag occurring?
    pub is_on: bool,
}

impl ArcBall {
    pub fn new(width: f32, height: f32) -> Self {
        ArcBall {
            win_size: (width, height),
            current: Point2::new(0.0, 0.0),
            click_vec: Vector3::new(0.0, 0.0, 0.0),
            drag_vec: Vector3::new(0.0, 0.0, 0.0),
            is_on: false,
            this_rot: Quaternion::new(1.0, 0.0, 0.0, 0.0),
            last_rot: Quaternion::new(1.0, 0.0, 0.0, 0.0),
        }
    }

    pub fn update(&mut self, width: f32, height: f32) {
        self.win_size.0 = width;
        self.win_size.1 = height;
    }

    pub fn click(&mut self, point: Point2<f32>) {
        self.current = point;
        self.is_on = true;
        self.click_vec = self.get_vector();
    }

    pub fn drag(&mut self, point: Point2<f32>) -> Quaternion<f32> {
        self.current = point;
        self.drag_vec = self.get_vector();

        // get the axis of rotation by crossing the click and drag vectors
        let perp = self.click_vec.cross(self.drag_vec);

        // since both vectors are normalized their dot will give us the angle of rotation
        self.this_rot = Quaternion::from_sv(self.click_vec.dot(self.drag_vec), perp);
        // Quaternion::from_axis_angle(perp, cgmath::Deg(self.click_vec.dot(self.drag_vec)));
        self.this_rot * self.last_rot
    }

    pub fn finish(&mut self) {
        self.is_on = false;
        self.last_rot = self.this_rot * self.last_rot;
    }

    pub fn reset(&mut self) {
        self.is_on = false;
        self.last_rot = Quaternion::new(1.0, 0.0, 0.0, 0.0);
        self.this_rot = Quaternion::new(1.0, 0.0, 0.0, 0.0);
    }

    // get the current vector to the center of the ball
    fn get_vector(&self) -> Vector3<f32> {
        let mut temp = self.current;

        // normalize the coordinates
        temp.x = 1.0 * temp.x / self.win_size.0 * 2.0 - 1.0;
        temp.y = -(1.0 * temp.y / self.win_size.1 * 2.0 - 1.0);

        // get the length of the vector
        let len = (temp.x * temp.x) + (temp.y * temp.y);

        if len <= 1.0 * 1.0 {
            Vector3::new(temp.x, temp.y, (1.0 * 1.0 - len).sqrt())
        } else {
            Vector3::new(temp.x, temp.y, 0.0).normalize()
        }
    }
}
