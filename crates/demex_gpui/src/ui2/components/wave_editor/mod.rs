use gpui::{
    App, Bounds, Context, Entity, EventEmitter, InteractiveElement, IntoElement, MouseDownEvent,
    MouseMoveEvent, MouseUpEvent, ParentElement, Path, PathBuilder, Pixels, RenderOnce,
    ScrollWheelEvent, Size, Styled, Window, canvas, div, fill, outline, point, px,
};

mod wave;
use gpui_component::{ActiveTheme, PixelsExt, black, white};
pub use wave::*;

use crate::ui2::ext::BoundsExt;

pub enum WaveEditorEvent {
    WaveSegmentValueChanged {
        /// The index of the segment within the wave
        segment_idx: usize,
        /// The index of the value within the segment values
        value_idx: usize,

        /// The new value
        value: f32,
    },

    WaveSegmentStartingPointChanged {
        /// The index of the segment within the wave
        segment_idx: usize,

        /// The new starting point
        starting_point: f32,
    },
}

#[derive(Default, Debug)]
pub struct WaveEditorState {
    wave: Wave,

    canvas_bounds: Option<Bounds<Pixels>>,

    keyframe_handle_bounds: Vec<Option<Bounds<Pixels>>>,
    current_dragging_keyframe_handle: Option<usize>,
}

impl EventEmitter<WaveEditorEvent> for WaveEditorState {}

impl WaveEditorState {
    pub fn new(wave: Wave) -> Self {
        Self {
            canvas_bounds: None,
            keyframe_handle_bounds: (0..wave.segments.len()).map(|_| None).collect(),
            current_dragging_keyframe_handle: None,
            wave,
        }
    }

    pub fn update_wave(&mut self, wave: Wave, cx: &mut Context<Self>) {
        self.wave = wave;

        self.keyframe_handle_bounds.clear();
        self.keyframe_handle_bounds
            .resize(self.wave.segments.len(), None);

        cx.notify();
    }

    fn update_canvas_bounds(&mut self, bounds: Bounds<Pixels>) {
        self.canvas_bounds = Some(bounds);
    }

    fn update_keyframe_handle_bounds(&mut self, idx: usize, bounds: Bounds<Pixels>) {
        self.keyframe_handle_bounds[idx] = Some(bounds);
    }

    fn handle_mouse_down(&mut self, evt: &MouseDownEvent, cx: &mut Context<Self>) {
        for (idx, bounds) in self
            .keyframe_handle_bounds
            .iter()
            .filter_map(|b| b.as_ref())
            .enumerate()
        {
            if bounds.contains(&evt.position) {
                self.current_dragging_keyframe_handle = Some(idx);
                cx.notify();
                return;
            }
        }

        if self.current_dragging_keyframe_handle.is_some() {
            self.current_dragging_keyframe_handle = None;
            cx.notify();
        }
    }

    fn handle_scroll_wheel(&mut self, evt: &ScrollWheelEvent, cx: &mut Context<Self>) {
        for (idx, bounds) in self
            .keyframe_handle_bounds
            .iter()
            .filter_map(|b| b.as_ref())
            .enumerate()
        {
            if bounds.contains(&evt.position) {
                let delta = evt.delta.pixel_delta(px(1.0)).y.as_f32() / 1000.0;

                let unmapped_x = (self.wave.segments[idx].starting_point + delta).clamp(0.0, 1.0);

                if idx > 0 {
                    let segment_below = &self.wave.segments[idx - 1];
                    if unmapped_x <= segment_below.starting_point {
                        return;
                    }
                }

                if idx < self.wave.segments.len() - 1 {
                    let segment_above = &self.wave.segments[idx + 1];
                    if unmapped_x >= segment_above.starting_point {
                        return;
                    }
                }

                self.wave.segments[idx].starting_point = unmapped_x;
                cx.emit(WaveEditorEvent::WaveSegmentStartingPointChanged {
                    segment_idx: idx,
                    starting_point: unmapped_x,
                });
                cx.notify();
                return;
            }
        }
    }

    fn handle_mouse_move(&mut self, evt: &MouseMoveEvent, cx: &mut Context<Self>) {
        let Some(canvas_bounds) = self.canvas_bounds else {
            return;
        };

        let Some(current_dragging) = self.current_dragging_keyframe_handle else {
            return;
        };

        let Some(unmapped_x) = canvas_bounds.unmap_x(evt.position.x).map(|p| p.as_f32()) else {
            return;
        };

        if current_dragging > 0 {
            let segment_below = &self.wave.segments[current_dragging - 1];
            if unmapped_x <= segment_below.starting_point {
                return;
            }
        }

        if current_dragging < self.wave.segments.len() - 1 {
            let segment_above = &self.wave.segments[current_dragging + 1];
            if unmapped_x >= segment_above.starting_point {
                return;
            }
        }

        self.wave.segments[current_dragging].starting_point = unmapped_x;

        cx.emit(WaveEditorEvent::WaveSegmentStartingPointChanged {
            segment_idx: current_dragging,
            starting_point: unmapped_x,
        });
        cx.notify();
    }

    fn handle_mouse_up(&mut self, _evt: &MouseUpEvent, cx: &mut Context<Self>) {
        if self.current_dragging_keyframe_handle.is_some() {
            self.current_dragging_keyframe_handle = None;
            cx.notify();
        }
    }
}

#[derive(IntoElement)]
pub struct WaveEditor {
    state: Entity<WaveEditorState>,
}

impl WaveEditor {
    pub fn new(state: &Entity<WaveEditorState>) -> Self {
        Self {
            state: state.clone(),
        }
    }
}

impl WaveEditor {
    const FOOTER_HEIGHT: Pixels = px(25.0);

    const PATH_STROKE_WIDTH: Pixels = px(2.0);

    fn paint_starting_point_marker(
        canvas_bounds: Bounds<Pixels>,
        starting_point: f32,
        window: &mut Window,
    ) -> Bounds<Pixels> {
        let mapped_starting_point = canvas_bounds.map_x(px(starting_point)).unwrap();
        let segment_point = point(
            mapped_starting_point,
            canvas_bounds.bottom() - Self::FOOTER_HEIGHT,
        );

        let triangle_x_offset = px(7.0);
        let triangle_height = px(10.0);

        let mut triangle_path = Path::new(segment_point);
        triangle_path.line_to(segment_point + point(-triangle_x_offset, triangle_height));
        triangle_path.line_to(segment_point + point(triangle_x_offset, triangle_height));

        window.paint_path(triangle_path, white());
        window.paint_quad(fill(
            Bounds::new(
                segment_point + point(-triangle_x_offset, triangle_height),
                Size::new(2 * triangle_x_offset, Self::FOOTER_HEIGHT - triangle_height),
            ),
            white(),
        ));

        Bounds::new(
            segment_point + point(-triangle_x_offset, px(0.0)),
            Size::new(2 * triangle_x_offset, Self::FOOTER_HEIGHT),
        )
    }

    fn paint_wave_segment(
        graph_bounds: Bounds<Pixels>,
        previous_segment: Option<&WaveSegment>,
        current_segment: &WaveSegment,
        is_last: bool,
        window: &mut Window,
        cx: &App,
    ) {
        let current_segment_start = graph_bounds
            .map_x(px(current_segment.starting_point))
            .unwrap();

        // TODO: find solution for this. we need to know which value of the previous segment
        // connects to which value of the current segment
        let current_segment_value = graph_bounds.map_y(px(current_segment.values[0])).unwrap();

        if let Some(previous_segment) = previous_segment {
            let previous_segment_start = graph_bounds
                .map_x(px(previous_segment.starting_point))
                .unwrap();

            for value in &previous_segment.values {
                let value_mapped = graph_bounds.map_y(px(*value)).unwrap();

                let mut path_builder = PathBuilder::stroke(Self::PATH_STROKE_WIDTH);

                let from = point(previous_segment_start, value_mapped);
                let to = point(current_segment_start, current_segment_value);

                path_builder.move_to(from);

                let easing_mode = previous_segment.easing_functions.get_easing_mode(from, to);

                match easing_mode {
                    WaveEasingMode::Cubic(a, b) => path_builder.cubic_bezier_to(to, a, b),
                    WaveEasingMode::Linear => {
                        path_builder.line_to(point(current_segment_start, current_segment_value))
                    }
                    WaveEasingMode::Snap => {
                        path_builder.line_to(point(current_segment_start, value_mapped));
                        path_builder.line_to(point(current_segment_start, current_segment_value));
                    }
                };

                if is_last {
                    path_builder.line_to(point(graph_bounds.right(), current_segment_value))
                }

                if let Ok(path) = path_builder.build() {
                    window.paint_path(path, cx.theme().blue);
                }
            }
        } else {
            // this means the current segment is the first segment
            // just draw a line from the start of the graph to the start of the current segment
            let mut path_builder = PathBuilder::stroke(Self::PATH_STROKE_WIDTH);
            path_builder.move_to(graph_bounds.bottom_left());
            path_builder.line_to(point(current_segment_start, graph_bounds.bottom()));
            path_builder.line_to(point(current_segment_start, current_segment_value));

            if let Ok(path) = path_builder.build() {
                window.paint_path(path, cx.theme().blue);
            }
        }
    }
}

impl RenderOnce for WaveEditor {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl gpui::IntoElement {
        div()
            .on_mouse_down(gpui::MouseButton::Left, {
                let state = self.state.clone();
                move |evt, _, cx| state.update(cx, |state, cx| state.handle_mouse_down(evt, cx))
            })
            .on_mouse_move({
                let state = self.state.clone();
                move |evt, _, cx| state.update(cx, |state, cx| state.handle_mouse_move(evt, cx))
            })
            .on_mouse_up(gpui::MouseButton::Left, {
                let state = self.state.clone();
                move |evt, _, cx| state.update(cx, |state, cx| state.handle_mouse_up(evt, cx))
            })
            .on_scroll_wheel({
                let state = self.state.clone();
                move |evt, _, cx| state.update(cx, |state, cx| state.handle_scroll_wheel(evt, cx))
            })
            .w_full()
            .child(
                canvas(
                    |_, _, _| {},
                    move |canvas_bounds, _, window, cx| {
                        self.state.update(cx, |state, _| {
                            state.update_canvas_bounds(canvas_bounds);
                        });

                        let mut graph_bounds = canvas_bounds;
                        graph_bounds.size.height -= Self::FOOTER_HEIGHT;

                        window.paint_quad(fill(graph_bounds, black()));
                        window.paint_quad(outline(graph_bounds, white(), gpui::BorderStyle::Solid));

                        if self.state.read(cx).wave.segments.is_empty() {
                            return;
                        }

                        let num_segments = self.state.read(cx).wave.segments.len();
                        // due to some conflict in itertools version (react-i18n grrrrr)
                        // i can't use tuple_windows() here
                        for idx in 0..num_segments {
                            let previous =
                                (idx != 0).then(|| &self.state.read(cx).wave.segments[idx - 1]);
                            let current = &self.state.read(cx).wave.segments[idx];

                            Self::paint_wave_segment(
                                graph_bounds,
                                previous,
                                current,
                                idx == num_segments - 1,
                                window,
                                cx,
                            );
                        }

                        // draw keyframe starting point handles
                        for (idx, segment) in self
                            .state
                            .read(cx)
                            .wave
                            .segments
                            .clone()
                            .into_iter()
                            .enumerate()
                        {
                            let bounds = Self::paint_starting_point_marker(
                                canvas_bounds,
                                segment.starting_point,
                                window,
                            );

                            self.state.update(cx, |state, _| {
                                state.update_keyframe_handle_bounds(idx, bounds);
                            });
                        }
                    },
                )
                .w_full()
                .h_56(),
            )
    }
}
