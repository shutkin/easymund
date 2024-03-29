use std::error::Error;
use std::fs;
use std::fs::File;
use std::path::PathBuf;
use log::info;

pub struct Ambience {
    pub id: String,
    pub name: String,
    pub data: Vec<f32>,
}

impl Ambience {
    pub fn read_dir(path: &str) -> Result<Vec<Ambience>, Box<dyn Error>> {
        let mut result = Vec::new();
        for entry in fs::read_dir(path)?.flatten() {
            let filename = entry.file_name().into_string().map_err(|s| format!("Invalid OsString {:?}", s))?;
            let sound_data = Ambience::read_sound(&entry.path(), 0.5)?;
            if let Some((id, name)) = filename.split_once('_') {
                let (name, _) = name.split_once('.').unwrap_or((name, ""));
                let ambience = Ambience {
                    id: String::from(id),
                    name: String::from(name),
                    data: sound_data,
                };
                info!("Read ambience id={}, name={}, length {}", &ambience.id, &ambience.name, ambience.data.len());
                result.push(ambience);
            }
        }
        Ok(result)
    }

    fn read_sound(path: &PathBuf, factor: f32) -> Result<Vec<f32>, Box<dyn Error>> {
        let mut background_file = File::open(path)?;
        let (_, bits) = wav::read(&mut background_file)?;
        let data_i16 = bits.try_into_sixteen().unwrap_or_default();
        Ok(data_i16.iter().map(|v| *v as f32 * factor / 32768.0).collect())
    }
}