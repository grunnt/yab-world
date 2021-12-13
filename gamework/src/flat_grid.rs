pub struct FlatGrid<T> {
    values: Vec<T>,
    width: usize,
    height: usize,
}

impl<T> FlatGrid<T>
where
    T: Clone,
{
    pub fn new(width: usize, height: usize, default: T) -> Self {
        FlatGrid {
            values: vec![default; width * height],
            width,
            height,
        }
    }

    pub fn get(&self, x: usize, y: usize) -> &T {
        &self.values[x + y * self.width]
    }

    pub fn set(&mut self, x: usize, y: usize, value: T) {
        self.values[x + y * self.width] = value;
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn values(&self) -> &Vec<T> {
        &self.values
    }
}
