import init, { WasmLib } from './easymund_client_processor.js';

class WasmProcessor extends AudioWorkletProcessor {
    lib = null;

    constructor() {
        super();
        this.port.onmessage = (e) => this.onmessage(e.data);
    }

    onmessage(event) {
        if (event.type === "wasm-module") {
            this.port.postMessage({type: "log", message: "WASM module received"});
            init(WebAssembly.compile(event.wasmBytes)).then(() => {
                this.lib = WasmLib.create();
                this.port.postMessage({type: "log", message: "WASM lib created"});
            });
        } else if (event.type === "stream") {
            this.lib.receive(event.data);
        }
    };

    onprocessorerror(err) {
        this.port.postMessage({type: "log", message: err});
    }

    process(inputList, outputList, parameters) {
        const input = inputList[0];
        const inputChannel = input[0];

        const output = outputList[0];
        const outputChannel0 = output[0];
        const outputChannel1 = output[1];

        if (this.lib) {
            const send = this.lib.process(inputChannel, outputChannel0);
            outputChannel1.set(outputChannel0);
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