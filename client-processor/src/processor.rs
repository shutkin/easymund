use std::collections::VecDeque;
use easymund_audio_codec::codec::{Codec, EasymundAudio};

pub struct Processor {
    packet_size: usize,
    buffer_in: VecDeque<f32>,
    buffer_out: VecDeque<f32>,
    codec: Codec,
}

impl Processor {
    pub fn create() -> Self {
        let packet_size = easymund_audio_codec::default_packet_size();
        let mut buffer_in = VecDeque::with_capacity(packet_size * 2);
        for _i in 0..packet_size / 2 {
            buffer_in.push_front(0.0);
        }
        let easymund_audio = EasymundAudio::new(44100, 1, 16);
        Self {
            packet_size,
            buffer_in,
            buffer_out: VecDeque::with_capacity(packet_size * 2),
            codec: easymund_audio.create_codec(packet_size).unwrap(),
        }
    }

    pub fn receive(&mut self, input: &[u8]) {
        match self.codec.decode(input) {
            Ok(decoded) => {
                for v in decoded[0].iter().copied() {
                    self.buffer_out.push_back(v);
                }
            }
            Err(e) => {
                let message = format!("Failed to decode: {:?}", e);
            }
        }
    }

    pub fn send(&mut self, output: &mut [u8]) -> usize {
        let mut buf = Vec::with_capacity(self.packet_size);
        for _ in 0..self.packet_size {
            buf.push(self.buffer_in.pop_front().unwrap_or_default());
        }

        match self.codec.encode(vec![buf.as_slice()].as_slice()) {
            Ok(encoded) => {
                output[..encoded.len()].copy_from_slice(encoded.as_slice());
                encoded.len()
            }
            Err(e) => {
                let message = format!("Failed to encode: {:?}", e);
                0
            }
        }
    }

    pub fn process(&mut self, input: &[f32], output: &mut [f32]) -> bool {
        for v in input.iter().copied() {
            self.buffer_in.push_back(v);
        }
        for i in 0..output.len() {
            output[i] = self.buffer_out.pop_front().unwrap_or_default();
        }
        self.buffer_in.len() >= self.packet_size
    }
}