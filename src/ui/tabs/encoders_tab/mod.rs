use egui::RichText;
use strum::IntoEnumIterator;

use crate::{
    fixture::channel3::{
        attribute::FixtureChannel3Attribute,
        channel_value::{FixtureChannelValue3, FixtureChannelValue3Discrete},
        feature::{
            feature_group::FixtureChannel3FeatureGroup, feature_type::FixtureChannel3FeatureType,
        },
    },
    ui::{
        components::{
            numpad::{numpad_ui, NumpadResult},
            tab_viewer::TabViewer,
        },
        constants::NO_FIXTURES_SELECTED,
        context::DemexUiContext,
    },
};

const NUM_ENCODERS: usize = 5;

#[derive(Debug, Default, Clone)]
pub struct ValueSelectionModalState {
    attribute: String,
    value: String,
}

#[derive(Debug, Default)]
pub struct EncodersTabState {
    pub feature: FixtureChannel3FeatureType,
    pub modal_state: Option<ValueSelectionModalState>,
    pub test_val: u8,
}

impl EncodersTabState {
    pub fn feature_group(&self) -> FixtureChannel3FeatureGroup {
        self.feature.feature_group()
    }

    pub fn attributes(&self) -> &[&'static str] {
        self.feature.attributes()
    }
}

pub fn ui(ui: &mut egui::Ui, context: &mut DemexUiContext) {
    if let (Some(mut modal_state), Some(fixtures)) = (
        context.encoders_tab_state.modal_state.clone(),
        context
            .global_fixture_select
            .as_ref()
            .map(|fixture_selection| fixture_selection.fixtures()),
    ) {
        egui::containers::Modal::new("EncodersTabModal".into()).show(ui.ctx(), |ui| {
            ui.heading(&modal_state.attribute);

            ui.horizontal(|ui| {
                numpad_ui(ui, &mut modal_state.value);

                ui.add_space(350.0);
            });

            ui.horizontal(|ui| {
                let mut fixture_handler = context.fixture_handler.write();
                let patch = context.patch.read();

                if ui.button("Ok").clicked() {
                    let numpad_result = NumpadResult::from_str(&modal_state.value)
                        .unwrap_or(NumpadResult::Value(0.0));

                    log::info!("result is: {:?}", numpad_result);
                    // apply values
                    //
                    for fixture_id in fixtures {
                        let fixture = fixture_handler.fixture(*fixture_id).unwrap();

                        let _ = fixture.update_programmer_attribute_value(
                            patch.fixture_types(),
                            &modal_state.attribute,
                            match numpad_result.clone() {
                                NumpadResult::Value(val) => {
                                    FixtureChannelValue3Discrete::Value(val)
                                }
                                NumpadResult::ChannelSet(channel_set) => {
                                    FixtureChannelValue3Discrete::ChannelSet(channel_set)
                                }
                            },
                        );
                    }

                    context.encoders_tab_state.modal_state = None;
                }

                if ui.button("Close").clicked() {
                    context.encoders_tab_state.modal_state = None;
                }

                if context.encoders_tab_state.modal_state.is_some() {
                    context.encoders_tab_state.modal_state = Some(modal_state);
                }
            });
        });
    }

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
        .auto_shrink(emath::Vec2b::FALSE)
        .show(ui, |ui| {
            ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                let mut encoder_size = ui.available_size();
                encoder_size.x /= NUM_ENCODERS as f32;

                if let Some(fixture_select) = context.global_fixture_select.as_ref() {
                    let mut fixture_handler = context.fixture_handler.write();
                    let preset_handler = context.preset_handler.read();
                    let timing_handler = context.timing_handler.read();
                    let patch = context.patch.read();

                    // TODO: find a better way, than to clone the attributes
                    let attributes = context.encoders_tab_state.attributes().to_vec();

                    for attribute in attributes {
                        let mut fixtures = fixture_handler.selected_fixtures_mut(fixture_select);

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

                                ui.horizontal(|ui| {
                                    ui.label(RichText::new(master_channel_functions[0].1).color(
                                        if !is_active {
                                            ecolor::Color32::PLACEHOLDER
                                        } else {
                                            ecolor::Color32::YELLOW
                                        },
                                    ));

                                    if ui.button("Sel").clicked() {
                                        context.encoders_tab_state.modal_state =
                                            Some(ValueSelectionModalState {
                                                attribute: attribute.to_string(),
                                                value: String::new(),
                                            });
                                    }
                                });

                                let (fixture_val_function_idx, fixture_val) = fixtures
                                    .iter()
                                    .find_map(|fixture| {
                                        fixture.get_programmer_value(master_channel_name).ok()
                                    }) // find first fixture, that has a value for the master_channel
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
                                                            ecolor::Color32::GREEN
                                                        } else {
                                                            ecolor::Color32::WHITE
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
                                        if should_home {
                                            for fixture in &mut fixtures {
                                                let _ = fixture.set_programmer_value(
                                                    patch.fixture_types(),
                                                    channel_name,
                                                    FixtureChannelValue3::Home,
                                                );
                                            }
                                        } else {
                                            let _ = fixtures[0].set_programmer_value(
                                                patch.fixture_types(),
                                                channel_name,
                                                FixtureChannelValue3::Discrete {
                                                    channel_function_idx: selected_channel_function,
                                                    value: slider_val,
                                                },
                                            );

                                            for fixture in fixtures.iter_mut().skip(1) {
                                                let _ = fixture.update_programmer_value(
                                                    patch.fixture_types(),
                                                    channel_name,
                                                    FixtureChannelValue3Discrete::Value(slider_val),
                                                );
                                            }
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
            });
        });
}
