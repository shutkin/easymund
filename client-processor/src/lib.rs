mod processor;

use wasm_bindgen::prelude::wasm_bindgen;
use crate::processor::Processor;

#[wasm_bindgen]
pub struct WasmLib {
    processor: Processor,
}

#[wasm_bindgen]
impl WasmLib {
    pub fn create() -> WasmLib {
        WasmLib {
            processor: Processor::create(),
        }
    }

    pub fn receive(&mut self, input: &[u8]) {
        self.processor.receive(input);
    }

    pub fn send(&mut self, output: &mut [u8]) -> usize {
        self.processor.send(output)
    }

    pub fn process(&mut self, input: &[f32], output: &mut [f32]) -> bool {
        self.processor.process(input, output)
    }
}
