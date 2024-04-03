export { EventBus }

class EventBus {
    constructor () {
        /**
         * @type Map<String, Function>
         */
        this.listeners = new Map();
    }

    /**
     * 
     * @param {String} event_type 
     * @param {Function} listener 
     */
    listen(event_type, listener) {
        this.listeners.set(event_type, listener);
    }

    /**
     * @param {Event} event 
     */
    fire(event) {
        const listener = this.listeners.get(event.type);
        if (listener) {
            listener(event);
        }
    }
}