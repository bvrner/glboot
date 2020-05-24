use cgmath::{InnerSpace, Point2, Quaternion, Vector3};

pub struct ArcBall {
    // scaling factor to normalize coordinates to viewport
    scales: (f32, f32),
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
        let mut ret = ArcBall {
            scales: (0.0, 0.0),
            current: Point2::new(0.0, 0.0),
            click_vec: Vector3::new(0.0, 0.0, 0.0),
            drag_vec: Vector3::new(0.0, 0.0, 0.0),
            is_on: false,
            this_rot: Quaternion::new(1.0, 0.0, 0.0, 0.0),
            last_rot: Quaternion::new(1.0, 0.0, 0.0, 0.0),
        };
        ret.update(width, height);

        ret
    }

    pub fn update(&mut self, width: f32, height: f32) {
        self.scales.0 = 1.0 / ((width - 1.0) * 0.5);
        self.scales.1 = 1.0 / ((height - 1.0) * 0.5);
    }

    pub fn click(&mut self, point: Point2<f32>) {
        self.current = point;
        self.is_on = true;
        self.this_rot = self.last_rot;
        self.click_vec = self.get_vector();
    }

    pub fn drag(&mut self, point: Point2<f32>) -> Quaternion<f32> {
        if self.is_on {
            self.current = point;
        }
        self.drag_vec = self.get_vector();

        // get the axis of rotation by crossing the click and drag vectors
        let perp = self.click_vec.cross(self.drag_vec);

        self.this_rot = if perp.magnitude() > f32::EPSILON {
            // since both vectors are normalized their dor will give us the angle of rotation
            Quaternion::from_sv(self.click_vec.dot(self.drag_vec), perp)
        } else {
            Quaternion::new(1.0, 0.0, 0.0, 0.0)
        };
        self.this_rot = self.this_rot * self.last_rot;
        self.this_rot
    }

    pub fn finish(&mut self) {
        self.is_on = false;
        self.last_rot = self.this_rot;
    }

    // get the current vector to the center of the ball
    fn get_vector(&self) -> Vector3<f32> {
        let mut temp = self.current;

        // normalize the coordinates
        temp.x = (temp.x * self.scales.0) - 1.0;
        temp.y = 1.0 - (temp.y * self.scales.1);

        // get the length of the vector
        let len = (temp.x * temp.x) + (temp.y * temp.y);

        if len > 1.0 {
            let norm = 1.0 / len.sqrt();

            Vector3::new(temp.x * norm, temp.y * norm, 0.0)
        } else {
            Vector3::new(temp.x, temp.y, (1.0 - len).sqrt())
        }
    }
}
