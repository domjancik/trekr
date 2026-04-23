use super::*;

impl App {
    pub(super) fn select_previous_page_item(&mut self) {
        if self.mapping_target_lookup_is_active() {
            self.move_mapping_target_lookup_highlight(-1);
            return;
        }
        self.clear_mapping_target_lookup();
        match self.page_state.current_page {
            AppPage::Timeline => {
                if self.page_state.selected_timeline_context == TimelineContext::TrackTimeline {
                    self.project.select_previous_track();
                } else {
                    self.select_timeline_fx_row(-1);
                }
            }
            AppPage::Mappings => {
                if !self.mappings.is_empty() {
                    let count = self.mappings.len();
                    self.page_state.selected_mapping_index =
                        (self.page_state.selected_mapping_index + count - 1) % count;
                    self.normalize_selected_mapping_field();
                    self.page_state.mapping_midi_learn_armed = false;
                }
            }
            AppPage::MidiIo => match self.page_state.midi_io.focus {
                MidiIoListFocus::Inputs => {
                    let count = self.midi_devices.inputs.len().max(1);
                    self.page_state.midi_io.selected_input_index =
                        (self.page_state.midi_io.selected_input_index + count - 1) % count;
                }
                MidiIoListFocus::Outputs => {
                    let count = self.midi_devices.outputs.len().max(1);
                    self.page_state.midi_io.selected_output_index =
                        (self.page_state.midi_io.selected_output_index + count - 1) % count;
                }
            },
            AppPage::Routing => {
                self.page_state.selected_routing_field =
                    self.page_state.selected_routing_field.previous();
            }
        }
    }

    pub(super) fn select_next_page_item(&mut self) {
        if self.mapping_target_lookup_is_active() {
            self.move_mapping_target_lookup_highlight(1);
            return;
        }
        self.clear_mapping_target_lookup();
        match self.page_state.current_page {
            AppPage::Timeline => {
                if self.page_state.selected_timeline_context == TimelineContext::TrackTimeline {
                    self.project.select_next_track();
                } else {
                    self.select_timeline_fx_row(1);
                }
            }
            AppPage::Mappings => {
                if !self.mappings.is_empty() {
                    self.page_state.selected_mapping_index =
                        (self.page_state.selected_mapping_index + 1) % self.mappings.len();
                    self.normalize_selected_mapping_field();
                    self.page_state.mapping_midi_learn_armed = false;
                }
            }
            AppPage::MidiIo => match self.page_state.midi_io.focus {
                MidiIoListFocus::Inputs => {
                    let count = self.midi_devices.inputs.len().max(1);
                    self.page_state.midi_io.selected_input_index =
                        (self.page_state.midi_io.selected_input_index + 1) % count;
                }
                MidiIoListFocus::Outputs => {
                    let count = self.midi_devices.outputs.len().max(1);
                    self.page_state.midi_io.selected_output_index =
                        (self.page_state.midi_io.selected_output_index + 1) % count;
                }
            },
            AppPage::Routing => {
                self.page_state.selected_routing_field =
                    self.page_state.selected_routing_field.next();
            }
        }
    }

    pub(super) fn select_previous_page_field(&mut self) {
        match self.page_state.current_page {
            AppPage::Timeline => self.select_previous_timeline_context(),
            AppPage::Mappings if self.page_state.mapping_mode == MappingPageMode::Write => {
                self.clear_mapping_target_lookup();
                self.page_state.selected_mapping_field =
                    self.previous_enabled_mapping_field(self.page_state.selected_mapping_field);
                self.page_state.mapping_midi_learn_armed = false;
            }
            _ => {}
        }
    }

    pub(super) fn select_next_page_field(&mut self) {
        match self.page_state.current_page {
            AppPage::Timeline => self.select_next_timeline_context(),
            AppPage::Mappings if self.page_state.mapping_mode == MappingPageMode::Write => {
                self.clear_mapping_target_lookup();
                self.page_state.selected_mapping_field =
                    self.next_enabled_mapping_field(self.page_state.selected_mapping_field);
                self.page_state.mapping_midi_learn_armed = false;
            }
            _ => {}
        }
    }

    pub(super) fn adjust_page_item(&mut self, delta: i32) {
        if self.mapping_target_lookup_is_active() {
            self.move_mapping_target_lookup_highlight(delta);
            return;
        }
        match self.page_state.current_page {
            AppPage::Timeline => self.adjust_timeline_context(delta),
            AppPage::Mappings => {
                if self.page_state.mapping_mode == MappingPageMode::Write
                    && !self.mappings.is_empty()
                {
                    self.clear_mapping_target_lookup();
                    self.adjust_mapping_field(delta);
                }
            }
            AppPage::MidiIo => {
                self.page_state.midi_io.focus = self.page_state.midi_io.focus.toggle();
            }
            AppPage::Routing => self.adjust_routing_field(delta),
        }
    }

    pub(super) fn activate_page_item(&mut self) {
        if self.mapping_target_lookup_is_active() {
            self.commit_mapping_target_lookup();
            return;
        }
        match self.page_state.current_page {
            AppPage::Timeline => self.activate_timeline_context_item(),
            AppPage::Mappings => {
                if self.page_state.mapping_mode == MappingPageMode::Write
                    && !self.mappings.is_empty()
                {
                    self.activate_mapping_field();
                }
            }
            AppPage::MidiIo => match self.page_state.midi_io.focus {
                MidiIoListFocus::Inputs => {
                    self.set_preferred_default_input_from_index(
                        self.page_state.midi_io.selected_input_index,
                    );
                }
                MidiIoListFocus::Outputs => self.set_preferred_default_output_from_index(
                    self.page_state.midi_io.selected_output_index,
                ),
            },
            AppPage::Routing => match self.page_state.selected_routing_field {
                RoutingField::Passthrough
                | RoutingField::RecordInputFx
                | RoutingField::MonitorInputFx
                | RoutingField::InputFxEnabled
                | RoutingField::OutputFxEnabled => self.adjust_routing_field(1),
                _ => {}
            },
        }
    }
}
