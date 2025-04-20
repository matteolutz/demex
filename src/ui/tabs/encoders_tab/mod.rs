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
    let mut fixture_handler = context.fixture_handler.write();
    let preset_handler = context.preset_handler.read();
    let timing_handler = context.timing_handler.read();
    let patch = context.patch.read();

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

    ui.allocate_ui_with_layout(
        ui.available_size(),
        egui::Layout::left_to_right(egui::Align::Center),
        |ui| {
            let mut encoder_size = ui.available_size();
            encoder_size.x /= NUM_ENCODERS as f32;

            if let Some(fixture_select) = context.global_fixture_select.as_ref() {
                for attribute in context.encoders_tab_state.attributes() {
                    let fixtures = fixture_handler.selected_fixtures_mut(fixture_select);

                    let channels = fixtures
                        .iter()
                        .flat_map(|fixture| {
                            fixture
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
                        })
                        .map(|(dmx_channel, _, attribute_name, channel_function_idx)| {
                            (
                                dmx_channel.name().as_ref().to_owned(),
                                attribute_name.to_owned(),
                                channel_function_idx,
                            )
                        })
                        .collect::<Vec<_>>();

                    if !channels.is_empty() {
                        let (
                            master_channel_name,
                            master_channel_attribute_name,
                            master_channel_function_idx,
                        ) = &channels[0];

                        let is_active = channels.iter().any(|(channel_name, _, _)| {
                            fixtures[0]
                                .get_programmer_value(channel_name.as_str())
                                .is_ok_and(|val| !val.is_home())
                        });

                        ui.vertical(|ui| {
                            ui.set_width(encoder_size.x);
                            ui.set_height(encoder_size.y);

                            ui.label(RichText::new(master_channel_attribute_name).color(
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
                                .ok()
                                .and_then(|val| {
                                    if val.0 != *master_channel_function_idx {
                                        None
                                    } else {
                                        Some(val)
                                    }
                                })
                                .unwrap_or_default();

                            let mut slider_val = fixture_val;

                            ui.add(egui::Slider::new(&mut slider_val, 0.0..=1.0));

                            if ui.button("Home").clicked() || slider_val != fixture_val {
                                for fixture in fixtures {
                                    for (channel_name, _, channel_function_idx) in &channels {
                                        let _ = fixture.set_programmer_value(
                                            patch.fixture_types(),
                                            channel_name,
                                            if slider_val != fixture_val {
                                                FixtureChannelValue3::Discrete {
                                                    channel_function_idx: *channel_function_idx,
                                                    value: slider_val,
                                                }
                                            } else {
                                                FixtureChannelValue3::Home
                                            },
                                        );
                                    }
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
        },
    );
}
