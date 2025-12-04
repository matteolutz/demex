use demex_core::layout::{FixtureLayoutEntry, FixtureLayoutEntryType};
use gpui::{App, BorderStyle, Bounds, PaintQuad, Window, px, size, transparent_black, white};
use gpui_component::{ActiveTheme, PixelsExt};

use crate::ui2::panels::layout_view::layout_projection::{LayoutProjection, PosExt};

pub(super) struct FixtureLayoutEntryDrawArgs {
    pub is_selected: bool,
}

pub(super) trait FixtureLayoutEntryExt {
    fn draw(
        &self,
        args: FixtureLayoutEntryDrawArgs,
        projection: &LayoutProjection,
        window: &mut Window,
        cx: &App,
    );
}

impl FixtureLayoutEntryExt for FixtureLayoutEntry {
    fn draw(
        &self,
        args: FixtureLayoutEntryDrawArgs,
        projection: &LayoutProjection,
        window: &mut Window,
        cx: &App,
    ) {
        let fixture_pos = projection.project(self.position().to_gpui_point(), cx);
        let fixture_size = projection.scale_size(size(px(5.0), px(5.0)));

        let stroke_color = if args.is_selected {
            // hsla(0.4, 1.0, 0.5, 1.0)
            cx.theme().green
        } else {
            white()
        };
        let stroke_width = 1.5;
        let fill_color = transparent_black();

        match self.entry_type() {
            FixtureLayoutEntryType::Rect => {
                window.paint_quad(PaintQuad {
                    bounds: Bounds::centered_at(fixture_pos, fixture_size),
                    corner_radii: (1.0).into(),
                    background: fill_color.into(),
                    border_widths: stroke_width.into(),
                    border_color: stroke_color,
                    border_style: BorderStyle::Solid,
                });
            }
            FixtureLayoutEntryType::Circle => {
                // let fixture_size = fixture_size / 2.0;
                let radius = fixture_size.width.as_f32() / 2.0; // fixture size is already projected

                window.paint_quad(PaintQuad {
                    bounds: Bounds::centered_at(fixture_pos, fixture_size),
                    corner_radii: radius.into(),
                    background: fill_color.into(),
                    border_widths: stroke_width.into(),
                    border_color: stroke_color,
                    border_style: BorderStyle::Solid,
                });
            }
            FixtureLayoutEntryType::Triangle => {
                todo!("triangle layout drawing")
            }
        }
    }
}
