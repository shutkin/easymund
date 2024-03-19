pub mod codec;

pub fn default_packet_size() -> usize {
    2048
}

#[cfg(test)]
mod tests {
    use rand::Rng;
    use crate::codec::EasymundAudio;

    #[test]
    fn test_filtered_noise() {
        let packet_size = 2048;
        let audio = EasymundAudio::new(44100, 1, 16);
        let mut codec = audio.create_codec(packet_size).expect("Codec must be created");
        let mut value = 0.0;
        for _ in 0..16 {
            let mut packet_data = Vec::with_capacity(packet_size as usize);
            let mut rng = rand::thread_rng();
            for _ in 0..packet_size {
                let rnd: f32 = rng.gen();
                let rnd = -1.0 + 2.0 * rnd;
                value = value * 0.85 + rnd * 0.15;
                packet_data.push(value);
            }
            let encoded = codec.encode(&[packet_data.as_slice()]).expect("Success encode");
            println!("{} samples encoded to {} bytes", packet_size, encoded.len());

            let decoded = codec.decode(encoded.as_slice()).expect("Success decode");
            for (i, f0) in packet_data.iter().enumerate() {
                let f1 = decoded[0][i];
                assert!((f1 - f0).abs() < 2.0 / i16::MAX as f32);
            }
        }
    }
}
