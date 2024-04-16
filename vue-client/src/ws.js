import { event_bus } from "./event_bus";

export { EasymundSocket }

class EasymundSocket {
    /**
     * @public
     * @param {String} room_name 
     */
    constructor (room_name) {
        this.is_online = false;
        this.message_queue = [];
        this.socket = new WebSocket("wss://" + window.location.hostname + ":5665/" + room_name);
        this.socket.binaryType = "arraybuffer";
        this.socket.onopen = (_) => {
            console.log("WS connected");
            this.is_online = true;
            while (this.message_queue.length > 0) {
                this.send_text(this.message_queue.shift());
            }
        }
        this.socket.onclose = (_) => {
            console.log("WS close");
            this.is_online = false;
        }
        this.socket.onerror = (e) => {
            console.log("WS error", e);
            this.is_online = false;
        }
        this.socket.onmessage = (e) => {
            if (typeof e.data === "string") {
                console.log("Message " + e.data);
                event_bus.fire({type: "ws_json", data: JSON.parse(e.data)});
            } else {
                this.receive_frame(e.data);
            }
        }
    }

    /**
     * @public
     */
    close() {
        if (this.is_online) {
            this.socket.close();
            this.is_online = false;
        }
    }

    /**
     * @private
     * @param {Uint8Array} data 
     */
    send_frame(data) {
        if (this.is_online) {
            this.socket.send(data);
        }
    }

    /**
     * @private
     * @param {String} text 
     */
    send_text(text) {
        if (this.is_online) {
            console.log("WS send text " + text);
            this.socket.send(text);
        } else {
            this.message_queue.push(text);
        }
    }

    /**
     * @private
     * @param {ArrayBuffer} data 
     */
    receive_frame(data) {
        const first_byte = new Uint8Array(data, 0, 1);
        const frame_data = new Uint8Array(data, 1);
        if (first_byte[0] == 0) {
            event_bus.fire({type: "ws_audio", data: frame_data});
        } else {
            event_bus.fire({type: "ws_video", data: frame_data});
        }
    }

    /**
     * @public
     * @param {Event} event
     */
    send_message(event) {
        if (event.type === "audio") {
            const data = new Uint8Array(event.data.length + 1);
            data[0] = 0;
            data.set(event.data, 1);
            this.send_frame(data);
        } else if (event.type === "video") {
            const data = new Uint8Array(event.data.length + 1);
            data[0] = 1;
            data.set(event.data, 1);
            this.send_frame(data);
        } else if (event.type === "json") {
            this.send_text(JSON.stringify(event.data));
        }
    }
}
