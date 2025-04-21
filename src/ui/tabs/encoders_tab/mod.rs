use egui::RichText;
use strum::IntoEnumIterator;

use crate::{
    fixture::channel3::{
        attribute::FixtureChannel3Attribute,
        channel_value::FixtureChannelValue3,
        feature::{
            feature_group::FixtureChannel3FeatureGroup, feature_type::FixtureChannel3FeatureType,
        },
    },
    ui::{
        components::tab_viewer::TabViewer, constants::NO_FIXTURES_SELECTED, context::DemexUiContext,
    },
};

const NUM_ENCODERS: usize = 5;

#[derive(Debug, Default)]
pub struct EncodersTabState {
    pub feature: FixtureChannel3FeatureType,
}

impl EncodersTabState {
    pub fn feature_group(&self) -> FixtureChannel3FeatureGroup {
        self.feature.feature_group()
    }

    pub fn attributes(&self) -> &[&str] {
        self.feature.attributes()
    }
}

pub fn ui(ui: &mut egui::Ui, context: &mut DemexUiContext) {
    let feature_group_tabs = FixtureChannel3FeatureGroup::iter().collect::<Vec<_>>();
    let selected_feature_group = feature_group_tabs
        .iter()
        .position(|fg| *fg == context.encoders_tab_state.feature_group())
        .unwrap();

    let feature_group_tab_viewer = TabViewer::new_without_state(
        "EncodersTabFeatureGroup",
        feature_group_tabs,
        selected_feature_group,
    );

    let new_selected_feature_group = feature_group_tab_viewer.show(ui).selected_tab;
    if new_selected_feature_group != context.encoders_tab_state.feature_group() {
        context.encoders_tab_state.feature = new_selected_feature_group.default_feature();
    }

    let feature_tabs = context
        .encoders_tab_state
        .feature_group()
        .features()
        .collect::<Vec<_>>();
    let selected_feature = feature_tabs
        .iter()
        .position(|f| *f == context.encoders_tab_state.feature)
        .unwrap();

    let feature_tab_viewer =
        TabViewer::new_without_state("EncodersTabFeature", feature_tabs, selected_feature);
    context.encoders_tab_state.feature = feature_tab_viewer.show(ui).selected_tab;

    ui.separator();

    egui::ScrollArea::horizontal()
        .auto_shrink(egui::Vec2b::FALSE)
        .show(ui, |ui| {
            ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                let mut encoder_size = ui.available_size();
                encoder_size.x /= NUM_ENCODERS as f32;

                if let Some(fixture_select) = context.global_fixture_select.as_ref() {
                    let mut fixture_handler = context.fixture_handler.write();
                    let preset_handler = context.preset_handler.read();
                    let timing_handler = context.timing_handler.read();
                    let patch = context.patch.read();

                    for attribute in context.encoders_tab_state.attributes() {
                        let mut fixtures = fixture_handler.selected_fixtures_mut(fixture_select);

                        let channels = fixtures[0]
                            .channels_for_attribute_matches(
                                patch.fixture_types(),
                                |fixture_attribute_name| {
                                    FixtureChannel3Attribute::attribute_matches(
                                        fixture_attribute_name,
                                        attribute,
                                    )
                                },
                            )
                            .unwrap()
                            .into_iter()
                            .map(|(dmx_channel, _, channel_functions)| {
                                (dmx_channel.name().as_ref().to_owned(), channel_functions)
                            })
                            .collect::<Vec<_>>();

                        if !channels.is_empty() {
                            let (master_channel_name, master_channel_functions) = &channels[0];

                            let is_active = channels.iter().any(|(channel_name, _)| {
                                fixtures[0]
                                    .get_programmer_value(channel_name.as_str())
                                    .is_ok_and(|val| !val.is_home())
                            });

                            ui.vertical(|ui| {
                                ui.set_width(encoder_size.x);
                                ui.set_height(encoder_size.y);

                                ui.label(RichText::new(master_channel_functions[0].1).color(
                                    if !is_active {
                                        egui::Color32::PLACEHOLDER
                                    } else {
                                        egui::Color32::YELLOW
                                    },
                                ));

                                let (fixture_val_function_idx, fixture_val) = fixtures[0]
                                    .get_programmer_value(master_channel_name)
                                    .map(|val| {
                                        val.get_as_discrete(
                                            fixtures[0],
                                            patch.fixture_types(),
                                            master_channel_name,
                                            &preset_handler,
                                            &timing_handler,
                                        )
                                    })
                                    .unwrap_or_default();

                                let mut slider_val = fixture_val;

                                ui.add(egui::Slider::new(&mut slider_val, 0.0..=1.0));

                                let (should_home, selected_channel_function) = ui
                                    .horizontal(|ui| {
                                        let should_home = ui.button("Home").clicked();

                                        let mut selected_channel_function =
                                            fixture_val_function_idx;

                                        for (channel_function_idx, channel_function_attribute) in
                                            master_channel_functions
                                        {
                                            if ui
                                                .button(
                                                    RichText::new(
                                                        channel_function_attribute.to_owned(),
                                                    )
                                                    .color(
                                                        if *channel_function_idx
                                                            == selected_channel_function
                                                        {
                                                            egui::Color32::GREEN
                                                        } else {
                                                            egui::Color32::WHITE
                                                        },
                                                    ),
                                                )
                                                .clicked()
                                            {
                                                selected_channel_function = *channel_function_idx;
                                            }
                                        }

                                        (should_home, selected_channel_function)
                                    })
                                    .inner;

                                if selected_channel_function != fixture_val_function_idx {
                                    slider_val = 0.0;
                                }

                                if should_home
                                    || selected_channel_function != fixture_val_function_idx
                                    || slider_val != fixture_val
                                {
                                    for (channel_name, _) in &channels {
                                        let _ = fixtures[0].set_programmer_value(
                                            patch.fixture_types(),
                                            channel_name,
                                            if !should_home {
                                                FixtureChannelValue3::Discrete {
                                                    channel_function_idx: selected_channel_function,
                                                    value: slider_val,
                                                }
                                            } else {
                                                FixtureChannelValue3::Home
                                            },
                                        );
                                    }
                                }
                            });
                        }
                    }
                } else {
                    ui.centered_and_justified(|ui| {
                        ui.label(NO_FIXTURES_SELECTED);
                    });
                }
            });
        });
}
