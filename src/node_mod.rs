use libm::acosf;
use std::f32::consts::PI;

#[derive(Copy, Clone, Debug)]
pub struct Node {
    pub x: f32,
    pub y: f32,
}

impl Node {
    pub fn _eq(&self, other: &Node) -> bool {
         self.x == other.x && self.y == other.y
    }

    fn pow(&self, pow: i32) -> Node {
        Node {
            x: self.x.powi(pow),
            y: self.y.powi(pow),
        }
    }

    fn sub(&self, other: &Node) -> Node {
        Node {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }

    fn added(&self) -> f32 {
        self.x + self.y
    }

    pub fn distance(&self, other: &Node) -> f32 {
        let mut val = self.sub(other);
        val = val.pow(2);
        let distance = val.added();
        distance.sqrt()
    }

    pub fn angle(&self, one: &Node, other: &Node) -> f32 {
        let gegenkathete = one.distance(other);
        let ankathete = self.distance(one);
        let hypothenuse = self.distance(other);

        let cos_angle = (ankathete.powi(2) + hypothenuse.powi(2) - gegenkathete.powi(2))
            / (2f32 * ankathete * hypothenuse);

        let angle = acosf(cos_angle);

        let angle_degrees: f32 = angle * 180f32 / PI;

        angle_degrees
    }

    pub fn _make_key(&self) -> (i32, i32) {
        (self.x as i32, self.y as i32)
    }
}
