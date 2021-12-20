mod grid;
mod input;
mod position;
mod size;
mod value;
mod widget;

use crate::video::color::*;
pub use grid::*;
pub use input::{InputEvent, Key, MouseButton};
pub use position::Position;
pub use size::*;
use std::collections::HashMap;
pub use value::GuiValue;
pub use widget::{Widget, WidgetId, NO_WIDGET};

pub const GUI_LAYER_PANEL: u8 = 1;
pub const GUI_LAYER_BACKGROUND: u8 = 2;
pub const GUI_LAYER_FOREGROUND: u8 = 3;
pub const GUI_LAYER_TEXT: u8 = 4;
pub const GUI_LAYER_DEBUG: u8 = 5;

#[derive(Clone, Debug)]
pub struct GuiConfig {
    pub foreground_color: ColorRGBA,
    pub background_color: ColorRGBA,
    pub input_color: ColorRGBA,
    pub input_focus_color: ColorRGBA,
}

impl GuiConfig {
    pub fn default() -> GuiConfig {
        GuiConfig {
            foreground_color: ColorRGBA::from_u8(255, 255, 255, 225),
            background_color: ColorRGBA::from_u8(49, 49, 175, 200),
            input_color: ColorRGBA::from_u8(14, 84, 96, 155),
            input_focus_color: ColorRGBA::from_u8(35, 114, 94, 220),
        }
    }
}

pub struct Gui<T> {
    widgets: HashMap<WidgetId, Box<dyn Widget<T>>>,
    grids: HashMap<WidgetId, Grid>,
    root: WidgetId,
    focus: Option<WidgetId>,
    last_screen_size: Size,
    grids_need_layout: bool,
    config: GuiConfig,
}

impl<T> Gui<T> {
    pub fn new(columns: Vec<ColumnLayout>, rows: Vec<RowLayout>) -> Gui<T> {
        let mut grids = HashMap::new();
        let root_id = WidgetId::next();
        let root_grid = Grid::new(columns, rows);
        grids.insert(root_id, root_grid);
        Gui {
            widgets: HashMap::new(),
            grids,
            root: root_id,
            focus: None,
            last_screen_size: Size::zero(),
            grids_need_layout: true,
            config: GuiConfig::default(),
        }
    }

    pub fn config_mut(&mut self) -> &mut GuiConfig {
        &mut self.config
    }

    pub fn root_id(&self) -> WidgetId {
        self.root
    }

    pub fn get_value(&self, widget_id: &WidgetId) -> GuiValue {
        self.widgets.get(widget_id).unwrap().get_value()
    }

    pub fn set_value(&mut self, widget_id: &WidgetId, value: GuiValue) {
        let widget = self.widgets.get_mut(widget_id).unwrap();
        widget.set_value(value);
    }

    pub fn get_widget(&self, widget_id: &WidgetId) -> &Box<dyn Widget<T>> {
        self.widgets.get(&widget_id).unwrap()
    }

    pub fn get_widget_mut(&mut self, widget_id: &WidgetId) -> &mut Box<dyn Widget<T>> {
        self.widgets.get_mut(&widget_id).unwrap()
    }

    pub fn place(
        &mut self,
        grid_id: WidgetId,
        column: usize,
        row: usize,
        widget: Box<dyn Widget<T>>,
        alignment: CellAlignment,
    ) -> WidgetId {
        assert!(self.grids.contains_key(&grid_id));
        let widget_id = WidgetId::next();
        self.widgets.insert(widget_id, widget);
        self.grids
            .get_mut(&grid_id)
            .unwrap()
            .place(column, row, widget_id, alignment);
        self.grids_need_layout = true;
        widget_id
    }

    pub fn remove(&mut self, grid_id: WidgetId, column: usize, row: usize) {
        assert!(self.grids.contains_key(&grid_id));
        let widget_id = self.grids.get_mut(&grid_id).unwrap().clear(column, row);
        if widget_id != NO_WIDGET {
            self.widgets.remove(&widget_id);
        }
        self.grids_need_layout = true;
    }

    pub fn grid(
        &mut self,
        grid_id: WidgetId,
        column: usize,
        row: usize,
        columns: Vec<ColumnLayout>,
        rows: Vec<RowLayout>,
    ) -> WidgetId {
        assert!(self.grids.contains_key(&grid_id));
        let widget_id = WidgetId::next();
        let grid = Grid::new(columns, rows);
        self.grids.insert(widget_id, grid);
        self.grids
            .get_mut(&grid_id)
            .unwrap()
            .place(column, row, widget_id, CellAlignment::Fill);
        self.grids_need_layout = true;
        widget_id
    }

    pub fn update(
        &mut self,
        input_events: &Vec<InputEvent>,
        screen_size: Size,
        context: &mut T,
    ) -> Vec<GuiEvent> {
        if self.last_screen_size != screen_size {
            self.grids_need_layout = true;
        }
        if self.grids_need_layout {
            self.layout(screen_size, context);
            self.grids_need_layout = false;
        }
        let root = self.root;
        let mut gui_events = Vec::new();
        for input_event in input_events {
            gui_events.extend(self.grid_handle_event(&root, input_event, self.focus));
        }

        // Handle focus requests here
        for gui_event in &gui_events {
            match gui_event {
                GuiEvent::FocusRequested { widget_id } => {
                    self.focus = Some(*widget_id);
                    break;
                }
                _ => {}
            }
        }
        gui_events.retain(|e| match e {
            GuiEvent::FocusRequested { .. } => false,
            _ => true,
        });
        gui_events
    }

    fn grid_handle_event(
        &mut self,
        grid_id: &WidgetId,
        input_event: &InputEvent,
        focus: Option<WidgetId>,
    ) -> Vec<GuiEvent> {
        // Now handle the child input events and (optionally) get events from the gui back
        let mut gui_events = Vec::new();
        let (columns, rows) = self.grids.get(&grid_id).unwrap().columns_rows_clone();
        for r in 0..rows.len() {
            let row = rows.get(r).unwrap();
            for c in 0..columns.len() {
                let column = columns.get(c).unwrap();
                let cell = self.grids.get(&grid_id).unwrap().cell(c, r).clone();
                if cell.widget_id == NO_WIDGET {
                    continue;
                }
                // Repostion some events to relative coordinates of this cell
                let input_event = match *input_event {
                    InputEvent::MouseClick { x, y, button } => InputEvent::MouseClick {
                        x: x - column.x,
                        y: y - row.y,
                        button,
                    },
                    InputEvent::MouseMove { x, y, dx, dy } => InputEvent::MouseMove {
                        x: x - column.x,
                        y: y - row.y,
                        dx,
                        dy,
                    },
                    _ => input_event.clone(),
                };
                // Check if positioned events are inside this cell's widget
                let cell_size = Size::new(column.size, row.size);
                let event_position = input_event.try_get_position();
                if let Some(pos) = event_position {
                    if !cell.inside_widget(pos.x, pos.y, cell_size) {
                        continue;
                    }
                }
                let is_grid = self.grids.contains_key(&cell.widget_id);
                if is_grid {
                    // Propagate recursively to child grid
                    gui_events.extend(self.grid_handle_event(&cell.widget_id, &input_event, focus));
                } else {
                    let widget = self.widgets.get_mut(&cell.widget_id).unwrap();
                    let has_focus = if let Some(focus_id) = focus {
                        if focus_id == cell.widget_id {
                            true
                        } else {
                            false
                        }
                    } else {
                        false
                    };
                    // Handle focus clicks
                    if let InputEvent::MouseClick { .. } = input_event {
                        if widget.is_focusable() {
                            // Focusable widget gets focus on click
                            gui_events.push(GuiEvent::FocusRequested {
                                widget_id: cell.widget_id,
                            });
                        }
                    }
                    // Handle event in widget
                    if let Some(gui_event) = widget.event(cell.widget_id, &input_event, has_focus) {
                        gui_events.push(gui_event);
                    }
                }
            }
        }
        gui_events
    }

    fn layout(&mut self, screen_size: Size, context: &mut T) {
        let root = self.root;
        self.layout_grid(&root, screen_size, context);
    }

    fn layout_grid(&mut self, grid_id: &WidgetId, size: Size, context: &mut T) {
        self.layout_grid_rows_colums(size, grid_id);
        let (columns, rows) = self.grids.get(&grid_id).unwrap().columns_rows_clone();
        for r in 0..rows.len() {
            let row = rows.get(r).unwrap();
            for c in 0..columns.len() {
                let col = columns.get(c).unwrap();
                let widget_id = self.grids.get(&grid_id).unwrap().cell(c, r).widget_id;
                // Layout the child widget or grid
                if widget_id == NO_WIDGET {
                    continue;
                }
                let cell_size = Size::new(col.size, row.size);
                let is_grid = self.grids.contains_key(&widget_id);
                if is_grid {
                    // Layout child grid
                    self.layout_grid(&widget_id, cell_size, context);
                    self.grids
                        .get_mut(&grid_id)
                        .unwrap()
                        .cell_mut(c, r)
                        .set_widget_size(cell_size);
                } else {
                    // Layout widget
                    let widget = self.widgets.get_mut(&widget_id).unwrap();
                    let size = widget.layout(&cell_size, context);
                    self.grids
                        .get_mut(&grid_id)
                        .unwrap()
                        .cell_mut(c, r)
                        .set_widget_size(size);
                }
            }
        }
    }

    // Layout all rows and columns
    fn layout_grid_rows_colums(&mut self, size: Size, grid_id: &WidgetId) {
        let grid = self.grids.get_mut(&grid_id).unwrap();
        let (column_count, row_count) = grid.grid_size();
        // Calculate space used by fixed cells
        let mut fixed_w = 0.0;
        let mut total_weight_w = 0.0;
        for c in 0..column_count {
            let col = grid.column(c);
            if col.layout.fixed {
                fixed_w += col.layout.value;
            } else {
                total_weight_w += col.layout.value;
            }
        }
        let mut fixed_h = 0.0;
        let mut total_weight_h = 0.0;
        for r in 0..row_count {
            let row = grid.row(r);
            if row.layout.fixed {
                fixed_h += row.layout.value;
            } else {
                total_weight_h += row.layout.value;
            }
        }
        // Determine space remaining
        let available_width = size.width - (column_count - 1) as f32 * grid.spacing;
        let remaining_w = if fixed_w < available_width {
            available_width - fixed_w
        } else {
            0.0
        };
        let available_height = size.height - (row_count - 1) as f32 * grid.spacing;
        let remaining_h = if fixed_h < available_height {
            available_height - fixed_h
        } else {
            0.0
        };
        // Assign position and size to rows and columns
        let mut x = 0.0;
        for c in 0..column_count {
            if c > 0 {
                x += grid.spacing;
            }
            let col = grid.column_mut(c);
            col.x = x;
            if col.layout.fixed {
                col.size = col.layout.value;
            } else {
                col.size = (col.layout.value / total_weight_w) * remaining_w;
            }
            x += col.size;
        }
        let mut y = 0.0;
        for r in 0..row_count {
            if r > 0 {
                y += grid.spacing;
            }
            let row = grid.row_mut(r);
            row.y = y;
            if row.layout.fixed {
                row.size = row.layout.value;
            } else {
                row.size = (row.layout.value / total_weight_h) * remaining_h;
            }
            y += row.size;
        }
    }

    pub fn paint(&mut self, context: &mut T) {
        let root = self.root;
        let config = self.config.clone();
        self.paint_grid(&root, 0.0, 0.0, context, &config, self.focus, 0);
    }

    fn paint_grid(
        &mut self,
        grid_id: &WidgetId,
        x: f32,
        y: f32,
        context: &mut T,
        config: &GuiConfig,
        focus: Option<WidgetId>,
        depth: usize,
    ) {
        let (col_count, row_count) = self.grids.get(&grid_id).unwrap().grid_size();
        for row in 0..row_count {
            let row_def = self.grids.get(&grid_id).unwrap().row(row);
            for col in 0..col_count {
                let col_def = self.grids.get(&grid_id).unwrap().column(col);
                let cell = self.grids.get(&grid_id).unwrap().cell(col, row).clone();
                let cell_size = Size::new(col_def.size, row_def.size);
                if cell.widget_id == NO_WIDGET {
                    continue;
                }
                // Render the widget or grid
                let widget_pos = cell.widget_position(cell_size);
                let is_grid = self.grids.contains_key(&cell.widget_id);
                if is_grid {
                    self.paint_grid(
                        &cell.widget_id,
                        x + col_def.x + widget_pos.x,
                        y + row_def.y + widget_pos.y,
                        context,
                        config,
                        focus,
                        depth + 1,
                    );
                } else {
                    let widget = self.widgets.get_mut(&cell.widget_id).unwrap();
                    let has_focus = if let Some(focus_id) = focus {
                        if focus_id == cell.widget_id {
                            true
                        } else {
                            false
                        }
                    } else {
                        false
                    };
                    if !cell.widget_size(cell_size).is_zero() {
                        widget.paint(
                            x + col_def.x + widget_pos.x,
                            y + row_def.y + widget_pos.y,
                            cell.widget_size(cell_size),
                            context,
                            config,
                            has_focus,
                        );
                    }
                }
            }
        }
    }

    pub fn set_focus(&mut self, focus: Option<WidgetId>) {
        self.focus = focus;
    }
}

#[derive(Clone, Debug)]
pub enum GuiEvent {
    ButtonClicked { widget_id: WidgetId },
    ValueSelected { widget_id: WidgetId, value: usize },
    FocusRequested { widget_id: WidgetId },
}
