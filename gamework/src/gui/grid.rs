use std::sync::atomic::{AtomicU64, Ordering};

use crate::*;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum CellAlignment {
    Fill,
    Center,
    Top,
    TopRight,
    Right,
    BottomRight,
    Bottom,
    BottomLeft,
    Left,
    TopLeft,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GridCell {
    pub widget_id: WidgetId,
    pub widget_alignment: CellAlignment,
    widget_size: Size,
}

impl GridCell {
    pub fn widget_position(&self, cell_size: Size) -> Position {
        if self.widget_size.is_zero() || cell_size.is_zero() {
            return Position::zero();
        }
        let w = self.widget_size;
        let c = cell_size;
        match self.widget_alignment {
            CellAlignment::Center => Position::new(
                c.width / 2.0 - w.width / 2.0,
                c.height / 2.0 - w.height / 2.0,
            ),
            CellAlignment::Fill => Position::new(0.0, 0.0),
            CellAlignment::Top => Position::new(c.width / 2.0 - w.width / 2.0, 0.0),
            CellAlignment::TopRight => Position::new(c.width - w.width, 0.0),
            CellAlignment::Right => {
                Position::new(c.width - w.width, c.height / 2.0 - w.height / 2.0)
            }
            CellAlignment::BottomRight => Position::new(c.width - w.width, c.height - w.height),
            CellAlignment::Bottom => {
                Position::new(c.width / 2.0 - w.width / 2.0, c.height - w.height)
            }
            CellAlignment::BottomLeft => Position::new(0.0, c.height - w.height),
            CellAlignment::Left => Position::new(0.0, c.height / 2.0 - w.height / 2.0),
            CellAlignment::TopLeft => Position::new(0.0, 0.0),
        }
    }

    pub fn widget_size(&self, cell_size: Size) -> Size {
        match self.widget_alignment {
            CellAlignment::Fill => cell_size,
            _ => self.widget_size,
        }
    }
    pub fn set_widget_size(&mut self, size: Size) {
        self.widget_size = size;
    }

    pub fn inside_widget(&self, x: f32, y: f32, cell_size: Size) -> bool {
        let pos = self.widget_position(cell_size);
        let size = self.widget_size(cell_size);
        x >= pos.x && x < pos.x + size.width && y >= pos.y && y < pos.y + size.height
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct ColumnLayout {
    pub value: f32,
    pub fixed: bool,
}

pub fn fixed_col(size: f32) -> ColumnLayout {
    ColumnLayout {
        value: size,
        fixed: true,
    }
}

pub fn flex_col(weight: f32) -> ColumnLayout {
    ColumnLayout {
        value: weight,
        fixed: false,
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Column {
    pub layout: ColumnLayout,
    pub x: f32,
    pub size: f32,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct RowLayout {
    pub value: f32,
    pub fixed: bool,
}

pub fn fixed_row(size: f32) -> RowLayout {
    RowLayout {
        value: size,
        fixed: true,
    }
}

pub fn flex_row(weight: f32) -> RowLayout {
    RowLayout {
        value: weight,
        fixed: false,
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Row {
    pub layout: RowLayout,
    pub y: f32,
    pub size: f32,
}

#[derive(Debug)]
pub struct Grid {
    columns: Vec<Column>,
    rows: Vec<Row>,
    cells: Vec<Vec<GridCell>>,
    pub spacing: f32,
}

impl Grid {
    pub fn new(columns: Vec<ColumnLayout>, rows: Vec<RowLayout>) -> Grid {
        assert!(columns.len() > 0);
        assert!(rows.len() > 0);
        let mut cells = Vec::new();
        for y in 0..rows.len() {
            cells.push(Vec::new());
            for _ in 0..columns.len() {
                cells.get_mut(y).unwrap().push(GridCell {
                    widget_id: NO_WIDGET,
                    widget_alignment: CellAlignment::Center,
                    widget_size: Size::zero(),
                });
            }
        }
        let columns = columns
            .iter()
            .map(|layout| {
                assert!(
                    layout.value > 0.0,
                    "Weight or size of columns cannot be zero"
                );
                Column {
                    layout: *layout,
                    x: 0.0,
                    size: 0.0,
                }
            })
            .collect();
        let rows = rows
            .iter()
            .map(|layout| {
                assert!(layout.value > 0.0, "Weight or size of rows cannot be zero");
                Row {
                    layout: *layout,
                    y: 0.0,
                    size: 0.0,
                }
            })
            .collect();
        Grid {
            columns,
            rows,
            cells,
            spacing: 5.0,
        }
    }

    pub fn row(&self, row: usize) -> Row {
        *self.rows.get(row).unwrap()
    }

    pub fn row_mut(&mut self, row: usize) -> &mut Row {
        self.rows.get_mut(row).unwrap()
    }

    pub fn column(&self, column: usize) -> Column {
        *self.columns.get(column).unwrap()
    }

    pub fn column_mut(&mut self, column: usize) -> &mut Column {
        self.columns.get_mut(column).unwrap()
    }

    pub fn columns_rows_clone(&self) -> (Vec<Column>, Vec<Row>) {
        (self.columns.clone(), self.rows.clone())
    }

    pub fn place(
        &mut self,
        column: usize,
        row: usize,
        widget_id: WidgetId,
        alignment: CellAlignment,
    ) {
        let cell = self.cell_mut(column, row);
        cell.widget_id = widget_id;
        cell.widget_alignment = alignment;
        cell.widget_size = Size::zero();
    }

    pub fn clear(&mut self, column: usize, row: usize) -> WidgetId {
        let cell = self.cell_mut(column, row);
        let old_widget_id = cell.widget_id;
        cell.widget_id = NO_WIDGET;
        cell.widget_alignment = CellAlignment::Center;
        cell.widget_size = Size::zero();
        old_widget_id
    }

    pub fn grid_size(&self) -> (usize, usize) {
        (self.columns.len(), self.rows.len())
    }

    pub fn cell(&self, column: usize, row: usize) -> &GridCell {
        self.cells.get(row).unwrap().get(column).unwrap()
    }

    pub fn cell_mut(&mut self, column: usize, row: usize) -> &mut GridCell {
        self.cells.get_mut(row).unwrap().get_mut(column).unwrap()
    }

    #[doc(hidden)]
    pub fn type_name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
}

pub const NO_GRID: GridId = GridId(0);
#[derive(Debug, Copy, Clone, PartialEq, Hash, Eq)]
pub struct GridId(u64);

impl GridId {
    pub fn next() -> GridId {
        static GRID_ID_COUNTER: AtomicU64 = AtomicU64::new(1);
        GridId(GRID_ID_COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}
