class ConferenceProcessor extends AudioWorkletProcessor {
    is_first = true;
    buffer_out = new Float32Array(0);
    index_out = 0;
    buffer_in = new Float32Array(0);
    index_in = 0;
    post_out = false;

    constructor() {
        super();
        this.port.onmessage = (e) => {
            if (this.is_first) {
                const silence = new Float32Array(e.data.length / 2);
                const newBuffer = new Float32Array(silence.length + e.data.length);
                newBuffer.set(silence, 0);
                newBuffer.set(e.data, silence.length);
                for (let i = 0; i < e.data.length; i++) {
                    newBuffer[i + silence.length] = newBuffer[i + silence.length] * i / e.data.length;
                }
                this.buffer_in = newBuffer;
                this.is_first = false;
            } else {
                const newBuffer = new Float32Array(this.buffer_in.length + e.data.length);
                newBuffer.set(this.buffer_in, 0);
                newBuffer.set(e.data, this.buffer_in.length);
                this.buffer_in = newBuffer;
                this.post_out = true;
            }
        }
    }
  
    process(inputList, outputList, parameters) {
        const input = inputList[0];
        const inputChannel = input[0];

        const newBuffer = new Float32Array(this.buffer_out.length + inputChannel.length);
        newBuffer.set(this.buffer_out, 0);
        newBuffer.set(inputChannel, this.buffer_out.length);
        this.buffer_out = newBuffer;
        if (this.post_out) {
            this.port.postMessage(this.buffer_out);
            this.buffer_out = new Float32Array(0);
            this.index_out = 0;
            this.post_out = true;
        }

        const output = outputList[0];
        const outputChannel0 = output[0];
        const outputChannel1 = output[1];
        let from = this.index_in;
        let to = from + outputChannel0.length;
        if (to > this.buffer_in.length) {
            to = this.buffer_in.length;
        }
        if (to > this.index_in) {
            outputChannel0.set(this.buffer_in.subarray(from, to));
            outputChannel1.set(this.buffer_in.subarray(from, to));

            const newBuffer = this.buffer_in.subarray(to);
            this.buffer_in = newBuffer;
            this.index_in = 0;
        }
        
        return true;
    }
}

registerProcessor("conference-processor", ConferenceProcessor);
