#[derive(Debug)]
pub enum GuiValue {
    String(String),
    Integer(i32),
    Usize(usize),
    Float(f32),
    Boolean(bool),
    None,
}
