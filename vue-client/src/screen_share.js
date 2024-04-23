import { event_bus } from "./event_bus";

export { EasymundScreenShare }

const mime_codecs = 'video/webm;codecs=vp9'

class EasymundScreenShare {
    constructor() {
        this.record_stream = null;
        this.recorder = null;
        this.play_source = null;
        this.source_buffer = null;
        this.next_video_frame = null;
    }

    async start_share() {
        try {
            const video_element = document.getElementById("video");
            const display_options = {
                video: {
                    displaySurface: "window",
                },
                audio: false,
                selfBrowserSurface: "exclude"
            };
            this.record_stream = await navigator.mediaDevices.getDisplayMedia(display_options);
            video_element.srcObject = this.record_stream;
            const track = this.record_stream.getVideoTracks()[0];
                console.log("Track settings:");
            console.log(JSON.stringify(track.getSettings()));

            const options = { mimeType: mime_codecs };
            this.recorder = new MediaRecorder(this.record_stream, options);
            this.recorder.ondataavailable = this.on_video_data;
            this.recorder.start(500);
        } catch (err) {
            console.error(err);
        }    
    }

    stop_share() {
        const video_element = document.getElementById("video");
        if (video_element) {
            video_element.srcObject = null;
        }
        if (this.recorder) {
            this.recorder.stop();
            this.recorder = null;
        }
        if (this.record_stream) {
            let tracks = this.record_stream.getTracks();
            tracks.forEach((track) => track.stop());
            this.record_stream = null;
        }
    }

    video_frame(data) {
        if (this.play_source == null) {
            this.next_video_frame = data;
            this.play_source = new MediaSource();
            const video_element = document.getElementById("video");
            video_element.src = URL.createObjectURL(this.play_source);
            this.play_source.onsourceopen = (_) => {
                this.source_buffer = this.play_source.addSourceBuffer(mime_codecs);
                console.log("Source buffer created");
                this.source_buffer.onerror = (e) => {
                    console.log("Media Source error: " + e.type);
                    this.play_source = null;
                }
                if (this.next_video_frame != null) {
                    this.source_buffer.appendBuffer(this.next_video_frame);
                    this.next_video_frame = null;
                }
            }
        } else if (this.source_buffer != null) {
            this.next_video_frame = data;
            this.source_buffer.appendBuffer(this.next_video_frame);
        }
    }

    /**
     * @private
     * @param {Event} event 
     */
    async on_video_data(event) {
        if (event.data.size > 0) {
            const buf = await event.data.arrayBuffer();
            event_bus.fire({type: "video_stream", data: new Uint8Array(buf)});
        }
    }
}