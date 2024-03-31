import init, { WasmLib } from './easymund_client_processor.js';

class WasmProcessor extends AudioWorkletProcessor {
    constructor() {
        super();
        this.lib = null;
        this.port.onmessage = (e) => this.onmessage(e.data);
        this.is_muted = false;
    }

    onmessage(event) {
        if (event.type === "wasm-module") {
            this.port.postMessage({type: "log", message: "WASM module received"});
            init(WebAssembly.compile(event.data)).then(() => {
                this.lib = WasmLib.create();
                this.port.postMessage({type: "log", message: "WASM lib created"});
            });
        } else if (event.type === "stream") {
            this.lib.receive(event.data);
        } else if (event.type === "mute") {
            this.is_muted = event.value;
            this.port.postMessage({type: "log", message: "is muted: " + this.is_muted});
        }
    }

    onerror(err) {
        this.port.postMessage({type: "log", message: err});
    }

    onprocessorerror(err) {
        this.port.postMessage({type: "log", message: err});
    }

    process(inputs, outputs, parameters) {
        const input = inputs[0];
        const input_channel = this.is_muted ? new Float32Array(input[0].length) : input[0];

        const output = outputs[0];
        const output_channel_0 = output[0];
        const output_channel_1 = output[1];

        if (this.lib) {
            const send = this.lib.process(input_channel, output_channel_0);
            output_channel_1.set(output_channel_0);
            if (send) {
                const buffer = new Uint8Array(4096);
                const size = this.lib.send(buffer);
                this.port.postMessage({type: "stream", data: buffer.subarray(0, size)});
            }
        }
        return true;
    }
}

registerProcessor("wasm-processor", WasmProcessor);