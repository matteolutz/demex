use demex_core::{
    fixture::{FixtureId, FixturePath},
    fpath,
    layout::{FixtureLayoutEntry, FixtureLayoutEntryType},
    selection::FixtureSelection,
};
use gpui::{
    App, BorderStyle, Bounds, Entity, Font, PaintQuad, Pixels, Point, Size, TextRun, Window, point,
    px, white,
};
use gpui_component::ActiveTheme;

use crate::{
    engine::state::DemexUiState,
    ui2::panels::layout_view::layout_projection::{LayoutProjection, PosExt},
};

#[derive(Debug)]
pub(super) struct FixtureLayoutEntryDrawEntry {
    pub entry_type: FixtureLayoutEntryType,
    pub fixture_path: FixturePath,
    pub pos: Point<Pixels>,
    pub size: Size<Pixels>,
}

#[derive(Clone)]
pub(super) struct FixtureLayoutEntryDrawArgs<'a> {
    pub selection: Option<&'a FixtureSelection>,
}

impl From<FixtureLayoutEntry> for FixtureLayoutEntryDrawEntry {
    fn from(value: FixtureLayoutEntry) -> Self {
        Self {
            entry_type: value.entry_type(),
            fixture_path: *value.fixture_path(),
            pos: value.position().to_gpui_point(),
            size: value.size().to_gpui_point().into(),
        }
    }
}

pub(super) trait FixtureLayoutEntryExt {
    fn get_draw_entries(&self) -> Vec<FixtureLayoutEntryDrawEntry>;

    fn draw(
        &self,
        args: FixtureLayoutEntryDrawArgs,
        projection: &Entity<LayoutProjection>,
        window: &mut Window,
        cx: &mut App,
    );
}

impl FixtureLayoutEntryExt for FixtureLayoutEntry {
    fn get_draw_entries(&self) -> Vec<FixtureLayoutEntryDrawEntry> {
        if let Some(line) = self.line() {
            let n_entries =
                line.to_fixture_id().as_u32() as i32 - self.fixture_path().last().as_u32() as i32;

            let fixture_pos = self.position().to_gpui_point();

            let mut entries = Vec::with_capacity(n_entries as usize);
            entries.push(self.clone().into());

            for (idx, f_id) in ((self.fixture_path().last().as_u32() + 1)
                ..=(line.to_fixture_id().as_u32()))
                .enumerate()
            {
                let mut fixture_path = *self.fixture_path();
                fixture_path.replace_last(FixtureId::new(f_id).unwrap());

                let pos = fixture_pos + (line.offset().to_gpui_point() * (idx as f32 + 1.0));

                entries.push(FixtureLayoutEntryDrawEntry {
                    entry_type: self.entry_type(),
                    fixture_path,
                    pos,
                    size: self.size().to_gpui_point().into(),
                });
            }

            entries
        } else {
            vec![self.clone().into()]
        }
    }

    fn draw(
        &self,
        args: FixtureLayoutEntryDrawArgs,
        projection: &Entity<LayoutProjection>,
        window: &mut Window,
        cx: &mut App,
    ) {
        let stroke_width = projection.read(cx).scale(0.5);

        let draw_entries = self.get_draw_entries();
        let master_pos = projection.read(cx).project(draw_entries[0].pos, cx);

        let name_fpath = if self.line().is_some() && !self.fixture_path().is_root_fixture() {
            fpath![self.fixture_path().root()]
        } else {
            *self.fixture_path()
        };
        let name = DemexUiState::patch(cx)
            .read(cx)
            .fixture(&name_fpath)
            .map(|f| f.name().to_string())
            .unwrap_or_else(|_| "[Deleted]".to_string());

        let name_len = name.len();

        let name_pos_offset = if let Some(line) = self.line() {
            point(px(0.0), px(if line.offset().y < 0.0 { 1.0 } else { -1.0 }))
        } else {
            point(px(0.0), px(1.0))
        } * projection.read(cx).scale(5.0);

        let shaped_line = window.text_system().shape_line(
            name.into(),
            px(projection.read(cx).scale(3.0)),
            &[TextRun {
                len: name_len,
                font: Font::default(),
                color: white(),
                background_color: None,
                underline: None,
                strikethrough: None,
            }],
            None,
        );

        let _ = shaped_line.paint(
            master_pos + name_pos_offset,
            px(0.0),
            gpui::TextAlign::Center,
            // Some(projection.read(cx).unscale(shaped_line.width))
            Some(shaped_line.width),
            window,
            cx,
        );

        let projection = projection.read(cx);

        for FixtureLayoutEntryDrawEntry {
            entry_type,
            fixture_path,
            pos,
            size,
        } in draw_entries
        {
            let stroke_color = if args
                .selection
                .is_some_and(|fs| fs.has_fixture(&fixture_path))
            {
                // hsla(0.4, 1.0, 0.5, 1.0)
                cx.theme().green
            } else {
                white()
            };

            let pos = projection.project(pos, cx);
            // let size = projection.scale_size(size(px(5.0), px(5.0)));
            let size = projection.scale_size(size);

            // TODO: find this
            let dimmer_value = Some(0.0);
            let fill_color = white().alpha(dimmer_value.unwrap_or(0.0));

            match entry_type {
                FixtureLayoutEntryType::Rect => {
                    window.paint_quad(PaintQuad {
                        bounds: Bounds::centered_at(pos, size),
                        corner_radii: (1.0).into(),
                        background: fill_color.into(),
                        border_widths: stroke_width.into(),
                        border_color: stroke_color.into(),
                        border_style: BorderStyle::Solid,
                    });
                }
                FixtureLayoutEntryType::Circle => {
                    // let fixture_size = fixture_size / 2.0;
                    let radius = size.width.as_f32() / 2.0; // fixture size is already projected

                    window.paint_quad(PaintQuad {
                        bounds: Bounds::centered_at(pos, size),
                        corner_radii: radius.into(),
                        background: fill_color.into(),
                        border_widths: stroke_width.into(),
                        border_color: stroke_color.into(),
                        border_style: BorderStyle::Solid,
                    });
                }
                FixtureLayoutEntryType::Triangle => {
                    todo!("triangle layout drawing")
                }
            }
        }
    }
}
