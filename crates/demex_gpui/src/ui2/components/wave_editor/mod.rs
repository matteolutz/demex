use std::rc::Rc;

use gpui::{
    App, Bounds, Context, Edges, Entity, EventEmitter, Font, InteractiveElement, IntoElement,
    MouseDownEvent, MouseMoveEvent, MouseUpEvent, ParentElement, Path, PathBuilder, Pixels, Point,
    RenderOnce, ScrollWheelEvent, Size, Styled, TextRun, Window, canvas, fill, outline, point, px,
};

mod wave;
use gpui_component::{ActiveTheme, PixelsExt, black, v_flex, white};
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

    WaveSegmentHandleRightClicked {
        /// The index of the segment within the wave
        segment_idx: usize,

        /// The mouse position when clicking
        click_pos: Point<Pixels>,
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

    pub fn segment_starting_points(&self) -> impl Iterator<Item = f32> {
        self.wave
            .segments
            .iter()
            .map(|segment| segment.starting_point)
    }

    pub fn set_segment_easing_function(
        &mut self,
        segment_idx: usize,
        curve: impl WaveEasingFunction,
        cx: &mut Context<Self>,
    ) {
        let Some(segment) = self.wave.segments.get_mut(segment_idx) else {
            return;
        };

        segment.easing_functions = Rc::new(curve);
        cx.notify();
    }

    fn update_canvas_bounds(&mut self, bounds: Bounds<Pixels>) {
        self.canvas_bounds = Some(bounds);
    }

    fn update_keyframe_handle_bounds(&mut self, idx: usize, bounds: Bounds<Pixels>) {
        self.keyframe_handle_bounds[idx] = Some(bounds);
    }

    fn handle_left_mouse_down(&mut self, evt: &MouseDownEvent, cx: &mut Context<Self>) {
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

    fn handle_right_mouse_down(&mut self, evt: &MouseDownEvent, cx: &mut Context<Self>) {
        for (idx, bounds) in self
            .keyframe_handle_bounds
            .iter()
            .filter_map(|b| b.as_ref())
            .enumerate()
        {
            if bounds.contains(&evt.position) {
                cx.emit(WaveEditorEvent::WaveSegmentHandleRightClicked {
                    segment_idx: idx,
                    click_pos: evt.position,
                });
                return;
            }
        }
    }

    fn handle_scroll_wheel(&mut self, evt: &ScrollWheelEvent, cx: &mut Context<Self>) {
        for (idx, bounds) in self
            .keyframe_handle_bounds
            .iter()
            .filter_map(|b| b.as_ref())
            .enumerate()
        {
            let extended_bounds = bounds.extend(Edges {
                top: px(0.0),
                bottom: px(0.0),
                left: px(10.0),
                right: px(10.0),
            });

            if extended_bounds.contains(&evt.position) {
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

    fn paint_graph_background(
        state: &Entity<WaveEditorState>,
        graph_bounds: Bounds<Pixels>,
        window: &mut Window,
        cx: &mut App,
    ) {
        window.paint_quad(fill(graph_bounds, black()));
        window.paint_quad(outline(graph_bounds, white(), gpui::BorderStyle::Solid));

        let phase_offset = state.read(cx).wave.phase_offset;
        let phase_length = state.read(cx).wave.phase_length;

        let mut path_builder = PathBuilder::stroke(px(1.0));
        for i in 0..=4 {
            let starting_point = point(
                graph_bounds.left() + (i as f32 / 4.0) * graph_bounds.size.width,
                graph_bounds.bottom(),
            );
            path_builder.move_to(starting_point);
            path_builder.line_to(starting_point + point(px(0.0), -graph_bounds.size.height));

            let phase = phase_offset + (phase_length * (i as f32 / 4.0));
            let phase_deg = phase.to_degrees();

            let phase_text = format!("{:.0}°", phase_deg);
            let phase_text_len = phase_text.len();

            let shaped_line = window.text_system().shape_line(
                phase_text.into(),
                px(12.0),
                &[TextRun {
                    len: phase_text_len,
                    font: Font::default(),
                    color: cx.theme().colors.muted_foreground,
                    background_color: None,
                    underline: None,
                    strikethrough: None,
                }],
                None,
            );

            let text_offset = if i == 4 {
                point(-shaped_line.width - px(5.0), px(-12.0))
            } else {
                point(px(5.0), px(-12.0))
            };
            let _ = shaped_line.paint(
                starting_point + text_offset,
                px(0.0),
                gpui::TextAlign::Left,
                Some(shaped_line.width),
                window,
                cx,
            );
        }
        window.paint_path(
            path_builder.build().unwrap(),
            cx.theme().colors.muted_foreground,
        );
    }

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
        v_flex()
            .on_mouse_down(gpui::MouseButton::Left, {
                let state = self.state.clone();
                move |evt, _, cx| {
                    state.update(cx, |state, cx| state.handle_left_mouse_down(evt, cx))
                }
            })
            .on_mouse_down(gpui::MouseButton::Right, {
                let state = self.state.clone();
                move |evt, _, cx| {
                    state.update(cx, |state, cx| state.handle_right_mouse_down(evt, cx))
                }
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
            .p_2()
            .child(
                canvas(|_, _, _| {}, {
                    let state = self.state.clone();
                    move |canvas_bounds, _, window, cx| {
                        state.update(cx, |state, _| {
                            state.update_canvas_bounds(canvas_bounds);
                        });

                        let mut graph_bounds = canvas_bounds;
                        graph_bounds.size.height -= Self::FOOTER_HEIGHT;

                        Self::paint_graph_background(&state, graph_bounds, window, cx);

                        if state.read(cx).wave.segments.is_empty() {
                            return;
                        }

                        graph_bounds = graph_bounds.inset(px(2.0));
                        let num_segments = state.read(cx).wave.segments.len();
                        // due to some conflict in itertools version (react-i18n grrrrr)
                        // i can't use tuple_windows() here
                        for idx in 0..num_segments {
                            let previous =
                                (idx != 0).then(|| &state.read(cx).wave.segments[idx - 1]);
                            let current = &state.read(cx).wave.segments[idx];

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
                        for (idx, segment) in
                            state.read(cx).wave.segments.clone().into_iter().enumerate()
                        {
                            let bounds = Self::paint_starting_point_marker(
                                canvas_bounds,
                                segment.starting_point,
                                window,
                            );

                            state.update(cx, |state, _| {
                                state.update_keyframe_handle_bounds(idx, bounds);
                            });
                        }
                    }
                })
                .w_full()
                .h_56(),
            )
    }
}
