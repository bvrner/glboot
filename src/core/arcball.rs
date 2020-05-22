use cgmath::{InnerSpace, Point2, Quaternion, Vector3};

pub struct ArcBall {
    scales: (f32, f32),
    start: Point2<f32>,
    current: Point2<f32>,
    click_vec: Vector3<f32>,
    drag_vec: Vector3<f32>,
    pub is_on: bool,
}

impl ArcBall {
    pub fn new(width: f32, height: f32) -> Self {
        let mut ret = ArcBall {
            scales: (0.0, 0.0),
            start: Point2::new(0.0, 0.0),
            current: Point2::new(0.0, 0.0),
            click_vec: Vector3::new(0.0, 0.0, 0.0),
            drag_vec: Vector3::new(0.0, 0.0, 0.0),
            is_on: false,
        };
        ret.update(width, height);

        ret
    }

    pub fn update(&mut self, width: f32, height: f32) {
        self.scales.0 = 1.0 / ((width - 1.0) * 0.5);
        self.scales.1 = 1.0 / ((height - 1.0) * 0.5);
    }

    pub fn click(&mut self, point: Point2<f32>) {
        self.start = point;
        self.current = point;
        self.is_on = true;
        self.click_vec = self.get_vector();
    }

    pub fn drag(&mut self, point: Point2<f32>) -> Quaternion<f32> {
        if self.is_on {
            self.current = point;
        }
        self.drag_vec = self.get_vector();

        let perp = self.click_vec.cross(self.drag_vec);

        self.start = self.current;
        if perp.magnitude() > f32::EPSILON {
            Quaternion::from_sv(self.click_vec.dot(self.drag_vec), perp)
        } else {
            Quaternion::new(0.0, 0.0, 0.0, 0.0)
        }
    }

    fn get_vector(&self) -> Vector3<f32> {
        let mut temp = self.current;

        temp.x = (temp.x * self.scales.0) - 1.0;
        temp.y = 1.0 - (temp.y * self.scales.1);

        let len = (temp.x * temp.x) + (temp.y * temp.y);

        if len > 1.0 {
            let norm = 1.0 / len.sqrt();

            Vector3::new(temp.x * norm, temp.y * norm, 0.0)
        } else {
            Vector3::new(temp.x, temp.y, (1.0 - len).sqrt())
        }
    }
}
