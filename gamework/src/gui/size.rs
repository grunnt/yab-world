#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Size {
    pub width: f32,
    pub height: f32,
}

impl Size {
    pub fn new(width: f32, height: f32) -> Self {
        Size { width, height }
    }

    pub fn zero() -> Self {
        Size {
            width: 0.0,
            height: 0.0,
        }
    }

    pub fn limit_to_constraints(&mut self, constraints: &SizeConstraints) {
        if self.width < constraints.min.width {
            self.width = constraints.min.width;
        } else if self.width > constraints.max.width {
            self.width = constraints.max.width;
        }
        if self.height < constraints.min.height {
            self.height = constraints.min.height;
        } else if self.height > constraints.max.height {
            self.height = constraints.max.height;
        }
    }

    pub fn is_zero(&self) -> bool {
        self.width == 0.0 || self.height == 0.0
    }

    pub fn geq(&self, other: Size) -> bool {
        self.width >= other.width && self.height >= other.height
    }
}

#[derive(Clone, Debug)]
pub struct SizeConstraints {
    pub min: Size,
    pub max: Size,
}

impl SizeConstraints {
    pub fn new(
        min_width: f32,
        min_height: f32,
        max_width: f32,
        max_height: f32,
    ) -> SizeConstraints {
        SizeConstraints {
            min: Size::new(min_width, min_height),
            max: Size::new(max_width, max_height),
        }
    }

    pub fn ensure_is_inside(&mut self, other: &SizeConstraints) {
        if self.min.width < other.min.width {
            self.min.width = other.min.width;
        }
        if self.min.height < other.min.height {
            self.min.height = other.min.height;
        }
        if self.max.width > other.max.width {
            self.max.width = other.max.width;
        }
        if self.max.height > other.max.height {
            self.max.height = other.max.height;
        }
    }
}
