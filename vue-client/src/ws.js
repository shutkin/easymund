export { EasymundSocket }

class EasymundSocket {
    /**
     * 
     * @param {String} room_name 
     * @param {Function} listener 
     */
    constructor (room_name, listener) {
        this.is_online = false;
        this.listener = listener;
        this.socket = new WebSocket("wss://" + window.location.hostname + ":5665/" + room_name);
        this.socket.binaryType = "arraybuffer";
        this.socket.onopen = (_) => {
            console.log("WS connected");
            this.is_online = true;
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
     * @param {ArrayBuffer} data 
     */
    receive_frame(data) {
        this.listener({type: "stream", data: new Uint8Array(data)});
    }

    /**
     * @public
     * @param {Event} event
     */
    send_message(event) {
        if (event.type === "stream") {
            this.send_frame(event.data);
        } else if (event.type === "log") {
            console.log("WS log: " + event.message);
        }
    }
}
