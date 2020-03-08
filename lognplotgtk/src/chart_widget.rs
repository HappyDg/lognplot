// Deals with drawing on the chart drawing area, as well as keyboard handling.

use gtk::prelude::*;

use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;

use crate::meta_metrics::MetricRecorder;
use crate::session::DashBoardItem;
use lognplot::chart::{Chart, Curve, CurveData};

use lognplot::geometry::Size;
use lognplot::render::{draw_chart, CairoCanvas, ChartLayout, ChartOptions};
use lognplot::render::{x_pixel_to_domain, x_pixels_to_domain, y_pixel_to_domain};
use lognplot::time::TimeStamp;
use lognplot::tsdb::DataChangeEvent;
use lognplot::tsdb::TsDbHandle;

pub struct ChartState {
    chart: Chart,
    chart_options: ChartOptions,
    db: TsDbHandle,
    color_wheel: Vec<String>,
    color_index: usize,
    tailing: Option<f64>,
    perf_tracer: MetricRecorder,
    drag: Option<(f64, f64)>,
    draw_area: gtk::DrawingArea,
    id: String,
}

/// category10 color wheel
///
/// See also: https://matplotlib.org/users/dflt_style_changes.html#colors-in-default-property-cycle
pub const CATEGORY10_COLORS: &[&str] = &[
    "#1F77B4", "#FF7F0E", "#2CA02C", "#D62728", "#9467BD", "#8C564B", "#E377C2", "#7F7F7F",
    "#BCBD22", "#17BECF",
];

impl ChartState {
    pub fn new(db: TsDbHandle, draw_area: gtk::DrawingArea, id: &str) -> Self {
        let chart = Chart::default();
        // let color_wheel = vec!["blue".to_string(), "red".to_string(), "green".to_string()];
        let color_wheel: Vec<String> = CATEGORY10_COLORS.iter().map(|s| (*s).to_string()).collect();

        ChartState {
            chart,
            chart_options: ChartOptions::default(),
            db: db.clone(),
            color_wheel,
            color_index: 0,
            tailing: None,
            perf_tracer: MetricRecorder::new(db),
            drag: None,
            draw_area,
            id: id.to_owned(),
        }
    }

    pub fn into_handle(self) -> ChartStateHandle {
        Rc::new(RefCell::new(self))
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn add_curve(&mut self, name: &str) {
        // self.chart.add_curve(Curve::new());
        if !self.chart.has_signal(name) {
            let tsdb_data = CurveData::trace(name, self.db.clone());
            let color = self.next_color();
            let curve2 = Curve::new(tsdb_data, &color);

            self.chart.add_curve(curve2);
            self.chart.autoscale();
            self.repaint();
        } else {
            info!("Signal {} is already shown", name);
        }
    }

    pub fn next_color(&mut self) -> String {
        let color = self.color_wheel[self.color_index].clone();
        self.color_index += 1;
        if self.color_index >= self.color_wheel.len() {
            self.color_index = 0;
        }
        color
    }

    pub fn clear_curves(&mut self) {
        debug!("Kill all signals!");
        self.disable_tailing();
        self.chart.clear_curves();
        self.repaint();
    }

    pub fn get_session_item(&self) -> DashBoardItem {
        (&self.chart).into()
    }

    pub fn set_session_item(&mut self, item: &DashBoardItem) {
        if let DashBoardItem::Graph { curves } = item {
            self.clear_curves();
            for curve in curves {
                self.add_curve(curve);
            }
        }
    }

    /// Handle data change event from database.
    pub fn handle_event(&self, event: &DataChangeEvent) {
        // Check if we must update the chart:
        let update = event
            .changed_signals
            .iter()
            .any(|n| self.chart.has_signal(n));
        if update {
            self.repaint();
        }
    }

    fn repaint(&self) {
        self.draw_area.queue_draw();
    }

    /// Initial drag action of the mouse
    pub fn start_drag(&mut self, x: f64, y: f64) {
        debug!("Drag start! {}, {} ", x, y);
        self.disable_tailing();
        self.drag = Some((x, y));
    }

    /// Update drag of the mouse
    pub fn move_drag(&mut self, size: Size, x: f64, y: f64) {
        self.disable_tailing();
        if let Some((prev_x, prev_y)) = self.drag {
            let dx = x - prev_x;
            let dy = y - prev_y;
            self.do_drag(size, dx, dy);
        }
        self.drag = Some((x, y));
    }

    /// Drag the plot by the given amount.
    fn do_drag(&mut self, size: Size, dx: f64, dy: f64) {
        debug!("Drag! {}, {} ", dx, dy);

        let mut layout = ChartLayout::new(size);
        layout.layout(&self.chart_options);

        let amount = x_pixels_to_domain(&layout, &self.chart.x_axis, dx);

        self.chart.pan_horizontal_absolute(-amount);
        // TODO: pan vertical as well?
        // TODO: idea: auto fit vertically?
        self.chart.fit_y_axis();
        // self.chart.pan_vertical(dy* 0.001);
    }

    pub fn zoom_fit(&mut self) {
        debug!("Autoscale!");
        self.disable_tailing();
        self.chart.autoscale();
        self.repaint();
    }

    pub fn zoom_in_vertical(&mut self) {
        debug!("Zoom in vertical");
        self.zoom_vertical(0.1);
    }

    pub fn zoom_out_vertical(&mut self) {
        debug!("Zoom out vertical");
        self.zoom_vertical(-0.1);
    }

    fn zoom_vertical(&mut self, amount: f64) {
        self.disable_tailing();
        self.chart.zoom_vertical(amount);
        self.repaint();
    }

    pub fn zoom_in_horizontal(&mut self, around: Option<(f64, Size)>) {
        debug!("Zoom in horizontal");
        self.zoom_horizontal(-0.1, around);
    }

    pub fn zoom_out_horizontal(&mut self, around: Option<(f64, Size)>) {
        debug!("Zoom out horizontal");
        self.zoom_horizontal(0.1, around);
    }

    fn zoom_horizontal(&mut self, amount: f64, around: Option<(f64, Size)>) {
        let around = around.map(|p| {
            let (pixel, size) = p;

            let mut layout = ChartLayout::new(size);
            layout.layout(&self.chart_options);

            x_pixel_to_domain(pixel, &self.chart.x_axis, &layout)
        });
        self.disable_tailing();
        self.chart.zoom_horizontal(amount, around);
        self.chart.fit_y_axis();
        self.repaint();
    }

    pub fn set_cursor(&mut self, loc: Option<((f64, f64), Size)>) {
        if let Some((pixel, size)) = loc {
            let mut layout = ChartLayout::new(size.clone());
            layout.layout(&self.chart_options);

            let timestamp = x_pixel_to_domain(pixel.0, &self.chart.x_axis, &layout);
            let value = y_pixel_to_domain(pixel.1, &self.chart.y_axis, &layout);
            let timestamp = TimeStamp::new(timestamp);
            self.chart.cursor = Some((timestamp, value));
        } else {
            self.chart.cursor = None;
        }
        self.repaint();
    }

    pub fn pan_left(&mut self) {
        debug!("pan left!");
        self.disable_tailing();
        self.chart.pan_horizontal_relative(-0.1);
        self.chart.fit_y_axis();
        self.repaint();
    }

    pub fn pan_right(&mut self) {
        debug!("Pan right!");
        self.disable_tailing();
        self.chart.pan_horizontal_relative(0.1);
        self.chart.fit_y_axis();
        self.repaint();
    }

    pub fn pan_up(&mut self) {
        debug!("pan up!");
        self.disable_tailing();
        self.chart.pan_vertical(-0.1);
        self.repaint();
    }

    pub fn pan_down(&mut self) {
        debug!("pan down!");
        self.disable_tailing();
        self.chart.pan_vertical(0.1);
        self.repaint();
    }

    pub fn zoom_to_last(&mut self, tail_duration: f64) {
        self.chart.zoom_to_last(tail_duration);
        self.chart.fit_y_axis();
        self.repaint();
    }

    pub fn enable_tailing(&mut self, tail_duration: f64) {
        self.tailing = Some(tail_duration);
    }

    pub fn disable_tailing(&mut self) {
        self.tailing = None;
    }

    pub fn do_tailing(&mut self) {
        if let Some(x) = self.tailing {
            self.zoom_to_last(x);
        }
    }

    fn on_scroll_event(&mut self, e: &gdk::EventScroll) -> Inhibit {
        debug!(
            "Scroll wheel event! {:?}, {:?}, {:?}",
            e,
            e.get_delta(),
            e.get_direction()
        );
        let size = get_size(&self.draw_area);
        let pixel_x_pos = e.get_position().0;
        let around = Some((pixel_x_pos, size));
        match e.get_direction() {
            gdk::ScrollDirection::Up => {
                self.zoom_in_horizontal(around);
            }
            gdk::ScrollDirection::Down => {
                self.zoom_out_horizontal(around);
            }
            gdk::ScrollDirection::Left => {
                self.pan_left();
            }
            gdk::ScrollDirection::Right => {
                self.pan_right();
            }
            _ => {}
        }
        Inhibit(false)
    }

    fn draw_on_canvas(&self, canvas: &cairo::Context) -> Inhibit {
        let size = get_size(&self.draw_area);

        // println!("Draw, width = {:?}, height= {:?}", width, height);
        canvas.set_font_size(14.0);
        let mut canvas2 = CairoCanvas::new(&canvas);

        let t1 = Instant::now();

        let mut layout = ChartLayout::new(size.clone());
        layout.layout(&self.chart_options);

        draw_chart(&self.chart, &mut canvas2, &layout, &self.chart_options);

        let t2 = Instant::now();
        let draw_duration = t2 - t1;
        trace!("Drawing time: {:?}", draw_duration);

        // TODO: re-enable this internal performance metric:
        let draw_seconds: f64 = draw_duration.as_secs_f64();
        self.perf_tracer.log_meta_metric(
            &format!("META_chart_render_time_{}", self.id),
            t1,
            draw_seconds,
        );

        // Focus indicator!
        let is_focus = self.draw_area.is_focus();
        if is_focus {
            let padding = 1.0;
            gtk::render_focus(
                &self.draw_area.get_style_context(),
                &canvas,
                padding,
                padding,
                size.width - 2.0 * padding,
                size.height - 2.0 * padding,
            );
        }

        Inhibit(false)
    }

    fn on_motion_event(&mut self, e: &gdk::EventMotion) -> Inhibit {
        let pos = e.get_position();
        debug!("Mouse motion! {:?}", pos);
        let size = get_size(&self.draw_area);

        self.set_cursor(Some(((pos.0, pos.1), size.clone())));

        if e.get_state().contains(gdk::ModifierType::BUTTON1_MASK) {
            self.move_drag(size, pos.0, pos.1);
        }
        self.repaint();

        Inhibit(false)
    }

    fn on_key(&mut self, key: &gdk::EventKey) -> Inhibit {
        self.disable_tailing();
        match key.get_keyval() {
            gdk::enums::key::Up | gdk::enums::key::w => {
                self.pan_up();
            }
            gdk::enums::key::Down | gdk::enums::key::s => {
                self.pan_down();
            }
            gdk::enums::key::Left | gdk::enums::key::a => {
                self.pan_left();
            }
            gdk::enums::key::Right | gdk::enums::key::d => {
                self.pan_right();
            }
            gdk::enums::key::i => {
                self.zoom_in_vertical();
            }
            gdk::enums::key::k => {
                self.zoom_out_vertical();
            }
            gdk::enums::key::KP_Add | gdk::enums::key::l => {
                self.zoom_in_horizontal(None);
            }
            gdk::enums::key::KP_Subtract | gdk::enums::key::j => {
                self.zoom_out_horizontal(None);
            }
            gdk::enums::key::Home | gdk::enums::key::Return => {
                self.zoom_fit();
            }
            gdk::enums::key::BackSpace => {
                self.clear_curves();
            }

            x => {
                println!("Key! {:?}", x);
            }
        };

        Inhibit(true)
    }
}

pub type ChartStateHandle = Rc<RefCell<ChartState>>;

pub fn setup_drawing_area(
    draw_area: gtk::DrawingArea,
    db: TsDbHandle,
    chart_id: &str,
) -> ChartStateHandle {
    // Always get mouse pointer motion:
    draw_area.add_events(gdk::EventMask::ENTER_NOTIFY_MASK);
    draw_area.add_events(gdk::EventMask::POINTER_MOTION_MASK);
    draw_area.add_events(gdk::EventMask::LEAVE_NOTIFY_MASK);

    let chart_state = ChartState::new(db, draw_area.clone(), chart_id).into_handle();

    // Connect draw event:
    draw_area.connect_draw(
        clone!(@strong chart_state => move |_, c| { chart_state.borrow().draw_on_canvas(c) } ),
    );

    // Connect drop event:
    let targets = vec![gtk::TargetEntry::new(
        super::mime_types::SIGNAL_NAMES_MIME_TYPE,
        gtk::TargetFlags::empty(),
        0,
    )];
    draw_area.drag_dest_set(gtk::DestDefaults::ALL, &targets, gdk::DragAction::COPY);

    draw_area.connect_drag_data_received(
        clone!(@strong chart_state => move |w, _dc, _x, _y, data, _info, _time| {
            let mime_payload: String = data.get_text().expect("Must work!!").to_string();
            if let Ok(signal_names) = serde_json::from_str::<Vec<String>>(&mime_payload) {
                info!("DROP {:?}", signal_names);
                for signal_name in signal_names {
                    chart_state
                    .borrow_mut()
                    .add_curve(&signal_name);
                }
                w.grab_focus();
            } else {
                error!("Error in drop action, could not plot mime data: {}", mime_payload);
            }
        }),
    );

    draw_area.connect_button_press_event(clone!(@strong chart_state => move |w, e| {
        let pos = e.get_position();
        debug!("Mouse press! {:?}", pos);
        chart_state.borrow_mut().start_drag(pos.0, pos.1);
        w.grab_focus();
        Inhibit(false)
    }));

    draw_area.connect_leave_notify_event(clone!(@strong chart_state => move |_, _| {
        debug!("Mouse leave!");
        chart_state.borrow_mut().set_cursor(None);
        Inhibit(false)
    }));

    draw_area.connect_motion_notify_event(clone!(@strong chart_state => move |_, e| {
        chart_state.borrow_mut().on_motion_event(e)
    }));

    draw_area.connect_scroll_event(clone!(@strong chart_state => move |_, e| {
        chart_state.borrow_mut().on_scroll_event(e)
    }));

    // Connect key event:
    draw_area.connect_key_press_event(
        clone!(@strong chart_state => move |_, k| { chart_state.borrow_mut().on_key(k) } ),
    );

    setup_tailing_timer(chart_state.clone());

    chart_state
}

fn get_size(drawing_area: &gtk::DrawingArea) -> Size {
    let width = drawing_area.get_allocated_width() as f64;
    let height = drawing_area.get_allocated_height() as f64;
    Size::new(width, height)
}

/// Setup a timer to implement tailing of signals.
fn setup_tailing_timer(chart_state: ChartStateHandle) {
    // Refreshing timer!
    let tick = move || {
        chart_state.borrow_mut().do_tailing();
        gtk::prelude::Continue(true)
    };
    gtk::timeout_add(100, tick);
}
