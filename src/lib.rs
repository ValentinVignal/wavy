use clack_plugin::{
    clack_export_entry,
    entry::{DefaultPluginFactory, SinglePluginEntry},
    host::{HostAudioProcessorHandle, HostMainThreadHandle, HostSharedHandle},
    plugin::{
        Plugin, PluginAudioProcessor, PluginDescriptor, PluginError, PluginMainThread, PluginShared,
    },
    prelude::PluginExtensions,
    process::{Audio, Events, PluginAudioConfiguration, Process, ProcessStatus},
};

pub struct Wavy;

impl Plugin for Wavy {
    type AudioProcessor<'a> = WavyAudioProcessor<'a>;
    type Shared<'a> = WavyShared;
    type MainThread<'a> = WavyMainThread<'a>;

    fn declare_extensions(builder: &mut PluginExtensions<Self>, shared: Option<&Self::Shared<'_>>) {
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
    shared: &'a WavyShared,
}

impl<'a> PluginAudioProcessor<'a, WavyShared, WavyMainThread<'a>> for WavyAudioProcessor<'a> {
    fn activate(
        host: HostAudioProcessorHandle<'a>,
        main_thread: &mut WavyMainThread<'a>,
        shared: &'a WavyShared,
        audio_config: PluginAudioConfiguration,
    ) -> Result<Self, PluginError> {
        todo!()
    }

    fn process(
        &mut self,
        process: Process,
        audio: Audio,
        events: Events,
    ) -> Result<ProcessStatus, PluginError> {
        todo!()
    }
}

pub struct WavyShared {}

impl<'a> PluginShared<'a> for WavyShared {}

pub struct WavyMainThread<'a> {
    shared: &'a WavyShared,
}

impl<'a> PluginMainThread<'a, WavyShared> for WavyMainThread<'a> {}

clack_export_entry!(SinglePluginEntry<Wavy>);
