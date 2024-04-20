use std::error::Error;

use flacenc::bitsink::{BitSink, MemSink};
use flacenc::component::{BitRepr, StreamInfo};
use flacenc::config::Encoder;
use flacenc::source::{Fill, FrameBuf};
use symphonia::core::audio::{AudioBuffer, Signal};
use symphonia::core::codecs::{CODEC_TYPE_FLAC, CodecParameters, Decoder, DecoderOptions};
use symphonia::core::formats::Packet;
use symphonia::default::codecs::FlacDecoder;

pub struct EasymundAudio {
    sample_rate: usize,
    channels: u8,
    bits_per_sample: u8,
}

pub struct Codec {
    decoder: FlacDecoder,
    encoder_config: Encoder,
    stream_info: StreamInfo,
    frame_buf: FrameBuf,
    packet_size: u16,
    channels: u8,
    frame_number: usize,
}

impl Codec {
    pub fn decode(&mut self, data: &[u8]) -> Result<Vec<Vec<f32>>, Box<dyn Error>> {
        let packet = Packet::new_from_slice(0, 0, 0, data);
        let buffer_ref = self.decoder.decode(&packet)?;
        let mut buffer: AudioBuffer<f32> = buffer_ref.make_equivalent();
        buffer_ref.convert(&mut buffer);
        let mut channels = Vec::with_capacity(self.channels as usize);
        for i in 0..self.channels as usize {
            let decoded_data = buffer.chan(i);
            channels.push(Vec::from(decoded_data));
        }
        Ok(channels)
    }

    pub fn encode(&mut self, data: &[&[f32]]) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut interleaved_data = Vec::with_capacity(self.channels as usize * self.packet_size as usize);
        for i in 0..self.packet_size as usize {
            for channel in data {
                interleaved_data.push(if i < channel.len() { (channel[i] * i16::MAX as f32) as i32 } else { 0 });
            }
        }
        self.frame_buf.fill_interleaved(interleaved_data.as_slice())?;
        let frame = flacenc::encode_fixed_size_frame(&self.encoder_config, &self.frame_buf, self.frame_number, &self.stream_info)
            .map_err(|e| format!("{:?}", e))?;

        let mut sink: MemSink<u8> = MemSink::with_capacity(frame.count_bits());
        frame.write(&mut sink)?;
        self.frame_number += 1;
        Ok(Vec::from(sink.as_slice()))
    }
}

impl EasymundAudio {
    pub fn new(sample_rate: usize, channels: u8, bits_per_sample: u8) -> Self {
        Self {sample_rate, channels, bits_per_sample}
    }

    pub fn create_codec(&self, packet_size: usize) -> Result<Codec, Box<dyn Error>> {
        let packet_size = packet_size as u16;
        let (encoder_config, stream_info, frame_buf) = self.create_encoder(packet_size)?;
        let decoder = self.create_decoder(packet_size)?;
        Ok(Codec {decoder, encoder_config, stream_info, frame_buf, packet_size, channels: self.channels, frame_number: 0})
    }

    fn write_flac_stream_info<S: BitSink>(&self, packet_size: u16, dest: &mut S) -> Result<(), Box<dyn Error>> {
        dest.write::<u16>(packet_size).map_err(|e| format!("{:?}", e))?;
        dest.write::<u16>(packet_size).map_err(|e| format!("{:?}", e))?;
        dest.write_lsbs(1024_u32, 24).map_err(|e| format!("{:?}", e))?;
        dest.write_lsbs(8192_u32, 24).map_err(|e| format!("{:?}", e))?;
        dest.write_lsbs(self.sample_rate as u32, 20).map_err(|e| format!("{:?}", e))?;
        dest.write_lsbs(self.channels - 1, 3).map_err(|e| format!("{:?}", e))?;
        dest.write_lsbs(self.bits_per_sample - 1, 5).map_err(|e| format!("{:?}", e))?;
        dest.write_lsbs(0_u64, 36).map_err(|e| format!("{:?}", e))?;
        dest.write_bytes_aligned(&[0; 16]).map_err(|e| format!("{:?}", e))?;
        Ok(())
    }

    fn create_decoder(&self, packet_size: u16) -> Result<FlacDecoder, Box<dyn Error>> {
        let mut format_sink = MemSink::new();
        self.write_flac_stream_info(packet_size, &mut format_sink)?;
        let format_bytes = format_sink.into_inner();
        let codec_params = CodecParameters {
            codec: CODEC_TYPE_FLAC,
            extra_data: Some(format_bytes.into_boxed_slice()),
            ..Default::default()
        };
        let options = DecoderOptions {verify: false};
        Ok(FlacDecoder::try_new(&codec_params, &options)?)
    }

    fn create_encoder(&self, packet_size: u16) -> Result<(Encoder, StreamInfo, FrameBuf), Box<dyn Error>> {
        let mut encoder_config = Encoder::default();
        encoder_config.block_sizes = vec![packet_size as usize];
        let stream_info = StreamInfo::new(self.sample_rate as usize,
                                          self.channels as usize,
                                          self.bits_per_sample as usize);
        let frame_buf = FrameBuf::with_size(self.channels as usize,
                                                             packet_size as usize);
        Ok((encoder_config, stream_info, frame_buf))
    }
}