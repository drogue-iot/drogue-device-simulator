// A wrapper to make Paho JavaScript work in an ESM WASM environment

// I HATE JAVASCRIPT!

export class Client {
    constructor(endpoint, clientId) {
        try {
            this.client = new Paho.MQTT.Client(endpoint, clientId);
        }
        catch(e) {
            console.log(e);
            throw e;
        }
    }

    connect(opts) {
        console.log("Connection options: ", opts);
        this.client.connect(opts);
    }

    disconnect() {
        this.client.disconnect();
    }

    get connected() {
        this.client.isConnected()
    }

    subscribe(filter, opts) {
        this.client.subscribe(filter, opts)
    }

    publish(topic, payload, qos, retained) {
        this.client.publish(topic, payload, qos, retained)
    }

    set onConnectionLost(handler) {
        this.client.onConnectionLost = handler;
    }

    set onMessageArrived(handler) {
        this.client.onMessageArrived = (msg) => {
            console.log(msg);
            handler(new Message(msg));
        };
    }
}

export class Message {
    constructor(msg) {
        this.msg = msg;
    }

    get topic () {
        return this.msg.topic;
    }

    get payloadBytes() {
        return this.msg.payloadBytes;
    }
}