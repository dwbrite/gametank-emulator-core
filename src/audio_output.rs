use alloc::boxed::Box;
use alloc::collections::VecDeque;
use alloc::vec::Vec;
use core::ops::IndexMut;
use dasp_graph::{Buffer, Input, NodeData};
use dasp_interpolate::linear::Linear;
use dasp_signal::Signal;
use klingt::{AudioNode, Klingt};
use klingt::nodes::effect::SlewLimiter;
use klingt::nodes::sink::{RtrbSink};
use log::{debug, error, trace};
use rtrb::{Consumer, Producer, RingBuffer};
use petgraph::prelude::NodeIndex;

pub struct GameTankSignal {
    buffer: Consumer<u8>,
}

impl GameTankSignal {
    pub fn new(buffer: Consumer<u8>) -> Self {
        Self {
            buffer,
        }
    }
}

impl Signal for GameTankSignal {
    type Frame = f32;

    fn next(&mut self) -> Self::Frame {
        if let Ok(sample) = self.buffer.pop() {
            (sample as f32 / 255.0) * 2.0 - 1.0
        } else {
            error!("FEED THE BUFFFEERRRRRR");
            0.0
        }
    }

    fn is_exhausted(&self) -> bool {
        self.buffer.slots() < 64
    }
}

#[derive(Debug)]
pub struct RtrbSource {
    output_buffer: Consumer<Buffer>
}

pub struct GameTankAudio {
    pub(crate) producer: Producer<u8>,

    klingt: Klingt<GTNode>,

    idx_in: NodeIndex,
    idx_out: NodeIndex,

    pub sink_output: Consumer<Buffer>,

    resampled: VecDeque<f32>,

    output_queue: Producer<Buffer>, // ring buffer for output buffers

    pub sample_rate: f64,
    converter: Box<dyn Signal<Frame = f32> + Send>,
}

impl GameTankAudio {
    pub fn new(sample_rate: f64, target_sample_rate: f64) -> Self {
        // caps out around 48kHz, but technically the system can go higher...
        let (input_producer, input_buffer) = RingBuffer::<u8>::new(1024); // Ring buffer to hold GameTank samples
        let (output_producer, output_consumer) = RingBuffer::<Buffer>::new(4096); // Ring buffer to hold output buffers
        let interp = Linear::new(0.0, 0.0);

        let signal = GameTankSignal::new(input_buffer);
        let converter = signal.from_hz_to_hz(interp, sample_rate, target_sample_rate);

        // create a new audio graph
        let mut klingt = Klingt::default();

        let (sink_producer, sink_consumer) = RingBuffer::<Buffer>::new(4096);
        let out_node = NodeData::new1(GTNode::RtrbSink(RtrbSink { output: sink_producer }));


        let gt_node = NodeData::new1(GTNode::GameTankSource(RtrbSource{ output_buffer: output_consumer }));
        let slew_node = NodeData::new1(GTNode::SlewLimiter(SlewLimiter::with_slew_limit(1.0/20.0)));

        let idx_in = klingt.add_node(gt_node);
        let idx_slew = klingt.add_node(slew_node);
        let idx_out = klingt.add_node(out_node);

        // update graph edges:
        klingt.add_edge(idx_in, idx_slew, ());
        klingt.add_edge(idx_slew, idx_out, ());

        Self {
            producer: input_producer,
            klingt,
            idx_in,
            idx_out,
            resampled: VecDeque::with_capacity(1024),
            output_queue: output_producer,
            sample_rate,
            converter: Box::new(converter),
            sink_output: sink_consumer,
        }
    }

    pub fn convert_to_output_buffers(&mut self) {
        while !self.converter.is_exhausted() {
            self.resampled.push_back(self.converter.next());
        }

        while self.resampled.len() >= 64 && self.output_queue.slots() >= 8 {
            if let Ok(chunk) = self.resampled.drain(..64).collect::<Vec<_>>().try_into() {
                let mut buf = Buffer::SILENT;
                for (b, v) in buf.iter_mut().zip::<[f32;64]>(chunk) {
                    *b = v;
                }
                self.output_queue.push(buf).unwrap()
            }
        }
    }

    pub fn process_audio(&mut self) {
        let mut ready_to_output = 0;
        if let GTNode::GameTankSource(src) = &mut self.klingt.index_mut(self.idx_in).node {
            ready_to_output = src.output_buffer.slots();
        }

        // Generate buffers in a loop
        while ready_to_output >= 4 {
            self.klingt.processor.process(&mut self.klingt.graph, self.idx_out);

            if let GTNode::GameTankSource(src) = &mut self.klingt.index_mut(self.idx_in).node {
                ready_to_output = src.output_buffer.slots();
            }
        }
    }
}

impl AudioNode for RtrbSource {
    fn process(&mut self, _inputs: &[Input], output: &mut [Buffer]) {
        let b = match self.output_buffer.pop() {
            Ok(buf) => { buf }
            Err(_) => { error!("FEED THE BUFFER"); Buffer::SILENT }
        };
        for buffer in output.iter_mut() {
            *buffer = b.clone();
        }
        debug!("processed rtrb source");
    }
}

#[enum_delegate::implement(AudioNode, pub trait AudioNode { fn process(&mut self, inputs: &[Input], output: &mut [Buffer]);})]
pub enum GTNode {
    GameTankSource(RtrbSource),
    RtrbSink(RtrbSink),
    SlewLimiter(SlewLimiter)
}