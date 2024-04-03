import { EventBus } from "./event_bus";

export { EasymundAudio }

class EasymundAudio {
    /**
     * 
     * @param {EventBus} event_bus
     */
    constructor (event_bus) {
        this.event_bus = event_bus;
        this.context = null;
        this.processor = null;
        this.source = null;
    }

    /**
     * @public
     */
    async init() {
        try {
            const response = await window.fetch("./easymund_client_processor_bg.wasm");
            const wasm_bytes = await response.arrayBuffer();
            console.log("Loaded wasm " + wasm_bytes.byteLength + " bytes");

            this.context = new AudioContext({sampleRate: 44100});
            const stream = await navigator.mediaDevices.getUserMedia({audio: {noiseSuppression: false, echoCancellation: false, autoGainControl: true}, video: false});
            console.log("Mic stream", stream);
            this.source = this.context.createMediaStreamSource(stream);
            await this.context.audioWorklet.addModule("wasm_processor.js");
            this.processor = new AudioWorkletNode(this.context, "wasm-processor");
            this.processor.port.onmessage = (e) => {this.on_processor_message(e.data)};
            this.processor.port.onmessageerror = (e) => {console.log(e)};
            this.processor.port.postMessage({type: "audio_wasm", data: wasm_bytes});
            this.processor.onprocessorerror = (err) => {console.log("Processor error: " + err);}
            console.log("Conference processor node", this.processor);
            this.source.connect(this.processor).connect(this.context.destination);
        } catch (error) {
            console.error(error);
            window.alert("Failed to start audio: " + error);
        }
    }

    close() {
        if (this.source != null) {
            this.source.disconnect();
            this.source = null;
            console.log("Audio source diconnected");
        }
        if (this.context != null) {
            this.context.close();
            this.context = null;
            console.log("Audio context closed");
        }
    }
    
    /**
     * @private
     * @param {Event} event 
     */
    on_processor_message(event) {
        this.event_bus.fire(event);
    }

    /**
     * @param {Event} event
     */
    send_message(event) {
        if (this.processor != null) {
            this.processor.port.postMessage(event);
        }
    }
}
