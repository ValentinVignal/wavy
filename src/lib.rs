use clack_extensions::{
    audio_ports::{
        AudioPortFlags, AudioPortInfo, AudioPortInfoWriter, AudioPortType, PluginAudioPorts,
        PluginAudioPortsImpl,
    },
    note_ports::{
        NoteDialect, NoteDialects, NotePortInfo, NotePortInfoWriter, PluginNotePorts,
        PluginNotePortsImpl,
    },
};
use clack_plugin::{
    clack_export_entry,
    entry::{DefaultPluginFactory, SinglePluginEntry},
    events::spaces::CoreEventSpace,
    host::{HostAudioProcessorHandle, HostMainThreadHandle, HostSharedHandle},
    plugin::{
        Plugin, PluginAudioProcessor, PluginDescriptor, PluginError, PluginMainThread, PluginShared,
    },
    process::{Audio, Events, PluginAudioConfiguration, Process, ProcessStatus},
    utils::ClapId,
};
mod oscillator;
use oscillator::{Oscillator, SineOscillator};

pub struct Wavy;

impl Plugin for Wavy {
    type AudioProcessor<'a> = WavyAudioProcessor<'a>;
    type Shared<'a> = WavyShared;
    type MainThread<'a> = WavyMainThread<'a>;

    fn declare_extensions(
        builder: &mut clack_plugin::prelude::PluginExtensions<Self>,
        shared: Option<&Self::Shared<'_>>,
    ) {
        builder
            .register::<PluginAudioPorts>()
            .register::<PluginNotePorts>();
    }
}

impl<'a> PluginAudioPortsImpl for WavyMainThread<'a> {
    fn count(&mut self, is_input: bool) -> u32 {
        if !is_input {
            1
        } else {
            0
        }
    }

    fn get(&mut self, index: u32, is_input: bool, writer: &mut AudioPortInfoWriter) {
        if !is_input && index == 0 {
            writer.set(&AudioPortInfo {
                id: ClapId::new(1),
                name: b"main",
                channel_count: 2,
                flags: AudioPortFlags::IS_MAIN,
                port_type: Some(AudioPortType::STEREO),
                in_place_pair: None,
            });
        }
    }
}

impl<'a> PluginNotePortsImpl for WavyMainThread<'a> {
    fn count(&mut self, is_input: bool) -> u32 {
        if is_input {
            1
        } else {
            0
        }
    }

    fn get(&mut self, index: u32, is_input: bool, writer: &mut NotePortInfoWriter) {
        if is_input && index == 0 {
            writer.set(&NotePortInfo {
                id: ClapId::new(1),
                name: b"main",
                preferred_dialect: Some(NoteDialect::Clap),
                supported_dialects: NoteDialects::CLAP,
            })
        }
    }
}

impl DefaultPluginFactory for Wavy {
    fn get_descriptor() -> PluginDescriptor {
        use clack_plugin::plugin::features::*;

        PluginDescriptor::new("com.valentinvignal.wavy", "Wavy")
            .with_vendor("Valentin Vignal")
            .with_features([INSTRUMENT, SYNTHESIZER, STEREO])
    }

    fn new_shared(host: HostSharedHandle) -> Result<Self::Shared<'_>, PluginError> {
        Ok(WavyShared {})
    }

    fn new_main_thread<'a>(
        host: HostMainThreadHandle<'a>,
        shared: &'a Self::Shared<'a>,
    ) -> Result<Self::MainThread<'a>, PluginError> {
        Ok(Self::MainThread { shared })
    }
}

pub struct WavyAudioProcessor<'a> {
    osc: Box<dyn Oscillator + Send>,
    shared: &'a WavyShared,
}

impl<'a> PluginAudioProcessor<'a, WavyShared, WavyMainThread<'a>> for WavyAudioProcessor<'a> {
    fn activate(
        host: HostAudioProcessorHandle<'a>,
        main_thread: &mut WavyMainThread<'a>,
        shared: &'a WavyShared,
        audio_config: PluginAudioConfiguration,
    ) -> Result<Self, PluginError> {
        Ok(Self {
            osc: Box::new(SineOscillator::new(audio_config.sample_rate as f32)),
            shared,
        })
    }
    fn process(
        &mut self,
        process: Process,
        mut audio: Audio,
        events: Events,
    ) -> Result<ProcessStatus, PluginError> {
        let mut output_port = audio
            .output_port(0)
            .ok_or(PluginError::Message("No output port"))?;

        let mut output_channels = output_port
            .channels()?
            .into_f32()
            .ok_or(PluginError::Message("Output is not f32"))?;

        // A bit of acrobatics to get simultaneous mutable references to both the left and right channels
        let mut split = output_channels.split_at_mut(1);
        let (left, right) = (
            split
                .0
                .channel_mut(0)
                .ok_or(PluginError::Message("Left channel not found"))?,
            split
                .1
                .channel_mut(0)
                .ok_or(PluginError::Message("Right channel not found"))?,
        );

        for batch in events.input.batch() {
            for event in batch.events() {
                match event.as_core_event() {
                    Some(CoreEventSpace::NoteOn(event)) => self.osc.handle_note_on(event),
                    Some(CoreEventSpace::NoteOff(event)) => self.osc.handle_note_off(event),
                    _ => {}
                }
            }

            let (left, right) = (
                &mut left[batch.sample_bounds()],
                &mut right[batch.sample_bounds()],
            );

            self.osc.process(left, right);
        }

        if self.osc.is_active() {
            Ok(ProcessStatus::Continue)
        } else {
            Ok(ProcessStatus::Sleep)
        }
    }
}

pub struct WavyShared {}

impl<'a> PluginShared<'a> for WavyShared {}

pub struct WavyMainThread<'a> {
    shared: &'a WavyShared,
}

impl<'a> PluginMainThread<'a, WavyShared> for WavyMainThread<'a> {}

clack_export_entry!(SinglePluginEntry<Wavy>);
