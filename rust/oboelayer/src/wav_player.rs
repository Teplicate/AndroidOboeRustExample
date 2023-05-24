use std::fs::File;
use std::io::{BufReader};
use std::ops::{Deref, DerefMut};
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use hound::{WavIntoSamples, WavReader};
use oboe::{AudioOutputCallback, AudioOutputStream, AudioOutputStreamSafe, AudioStream, AudioStreamAsync, AudioStreamBuilder, AudioStreamSafe, DataCallbackResult, IsFrameType, Mono, Output, PerformanceMode, SharingMode, StreamState};

extern crate android_logger;

use log::debug;

pub struct WavPlayer {
    stream: Option<AudioStreamAsync<Output, AudioHolder>>,
    transmitter: Option<Sender<SampleActions>>,
}

const DIV: f32 = 1.0 / 32768.0;

impl WavPlayer {
    fn init_stream(&mut self, file_path: String) {
        let (transmitter, receiver) = mpsc::channel::<SampleActions>();
        let mut p = AudioHolder::open_file(file_path);
        p.request_receiver = Some(receiver);
        let builder = AudioStreamBuilder::default();
        let r = builder
            .set_f32()
            .set_performance_mode(PerformanceMode::LowLatency)
            .set_sharing_mode(SharingMode::Shared)
            .set_mono()
            .set_sample_rate(44_100)
            .set_callback(p)
            .open_stream()
            .unwrap();

        self.transmitter = Some(transmitter);
        self.stream = Some(r);
        self.stream.as_mut().unwrap().start().unwrap();
    }

    pub fn play(&mut self, file_path: String) {
        let stream = self.stream.as_mut();
        match stream {
            Some(stream) => {
                match self.transmitter.as_ref().unwrap().send(SampleActions::Reset) {
                    Ok(_) => debug!("oboe_log: sent request"),
                    Err(e) => debug!("oboe_log: err sending req {}", e)
                }
                match stream.start() {
                    Ok(_) => { debug!("oboe_log: playing existing!") }
                    Err(e) => { debug!("oboe_log: error playing existing {}", e) }
                }
            }
            None => {
                debug!("oboe_log: initializing");
                self.init_stream(file_path)
            }
        }
    }

    pub const fn new() -> Self {
        WavPlayer {
            stream: None,
            transmitter: None,
        }
    }
}

pub struct AudioHolder {
    sample_rate: f32,
    samples: WavReader<BufReader<File>>,
    request_receiver: Option<Receiver<SampleActions>>,
}

impl AudioHolder {
    fn open_file(file_path: String) -> Self {
        debug!("oboe_log: opening file {file_path}");
        let path_clone = file_path.clone();
        let reader = match hound::WavReader::open(file_path) {
            Ok(reader) => reader,
            Err(_) => panic!("Error opening file {}", path_clone)
        };
        let format = reader.spec();
        debug!("oboe_log: file opened dur {} specs - sample_rate {}, bits_per_sample {}", reader.len(), format.sample_rate, format.bits_per_sample);

        Self {
            sample_rate: format.sample_rate as f32,
            samples: reader,
            request_receiver: None,
        }
    }
}

impl AudioOutputCallback for AudioHolder {
    type FrameType = (f32, Mono);

    fn on_audio_ready(&mut self,
                      _: &mut dyn AudioOutputStreamSafe,
                      audio_data: &mut [<Self::FrameType as IsFrameType>::Type]) -> DataCallbackResult {
        let mut callback_res = DataCallbackResult::Continue;
        match self.request_receiver.as_ref().unwrap().try_recv().unwrap_or(SampleActions::Idle) {
            SampleActions::Reset => {
                debug!("oboe_log: resetting from receiver");
                match self.samples.seek(0) {
                    Ok(_) => { debug!("oboe_log shifted to start from request") }
                    Err(e) => debug!("oboe_log error shifting to 0 from req {}", e)
                }
            }
            _ => {}
        };


        for frame in audio_data {
            let value = self.samples.samples::<i32>().next();
            let unpacked_value = match value {
                Some(v) => match v {
                    Ok(v) => {
                        v
                    }
                    Err(e) => {
                        debug!("oboe_log: no value on res {}",e );
                        callback_res = DataCallbackResult::Stop;
                        break;
                    }
                },
                None => {
                    match self.samples.seek(0) {
                        Ok(_) => { debug!("oboe_log shifted to start") }
                        Err(e) => debug!("oboe_log error shifting to 0 {}", e)
                    }
                    debug!("oboe_log: no value on in opt");
                    callback_res = DataCallbackResult::Stop;
                    break;
                }
            };

            let converted = (unpacked_value as f32) * DIV;
            *frame = converted;
        }

        callback_res
    }
}

enum SampleActions {
    Idle,
    Reset,
}