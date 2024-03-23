let audioContext = null;
let processor = null;
const state = {isOnline: false, isActive: false}

const socket = new WebSocket("wss://" + window.location.hostname + ":5665/dev-room");
socket.binaryType = "arraybuffer";
socket.onopen = (_) => {
    console.log("WS connected");
    state.isOnline = true;
}
socket.onclose = (_) => {
    console.log("WS close");
    state.isOnline = false;
}
socket.onclose = (e) => {
    console.log("WS error", e);
    state.isOnline = false;
}
socket.onmessage = (e) => {
    if (typeof e.data === "string") {
        console.log("Message " + e.data);
    } else {
        receiveFrame(e.data);
    }
}

function sendFrame(data) {
    if (state.isOnline) {
        console.log("Send frame " + data.length + " bytes");
        socket.send(data);
    }
}

function receiveFrame(data) {
    if (state.isActive) {
        const array = new Uint8Array(data);
        processor.port.postMessage({type: "stream", data: array});
    }
}

function onProcessorMessage(event) {
    if (event.type === "stream") {
        sendFrame(event.data);
    } else if (event.type === "log") {
        console.log("Processor log: " + event.message);
    }
}

async function start() {
    if (state.isActive) return;
    if (audioContext == null) {
        try {
            const response = await window.fetch("pack/easymund_client_processor_bg.wasm");
            const wasmBytes = await response.arrayBuffer();
            console.log("Loaded wasm " + wasmBytes.byteLength + " bytes");

            audioContext = new AudioContext({sampleRate: 44100});
            const stream = await navigator.mediaDevices.getUserMedia({audio: {noiseSuppression: false, echoCancellation: false, autoGainControl: true}, video: false});
            console.log("Mic stream", stream);
            const source = audioContext.createMediaStreamSource(stream);
            await audioContext.audioWorklet.addModule("wasm_processor.js");
            processor = new AudioWorkletNode(audioContext, "wasm-processor");
            processor.port.onmessage = (e) => {onProcessorMessage(e.data);};
            processor.port.onmessageerror = (e) => {console.log(e);};
            processor.port.postMessage({type: "wasm-module", wasmBytes});
            processor.onprocessorerror = (err) => {console.log("Processor error: " + err);}
            console.log("Conference processor node", processor);
            source.connect(processor).connect(audioContext.destination);
            state.isActive = true;
            document.getElementById("button_start").style.display = "none";
        } catch (error) {
            window.alert(error);
        }
    }
}
