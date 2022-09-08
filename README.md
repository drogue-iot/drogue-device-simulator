# Drogue IoT device simulator

This is an IoT device simulator, which runs fully in your web browser. It is intended to work with
[Drogue IoT Cloud](https://github.com/drogue-iot/drogue-cloud), but can work with any other MQTT endpoint.

There are publicly hosted versions of the simulator:

* v1 - https://v1.device-simulator.com
* latest - https://latest.device-simulator.com

## Versioning

`latest` will always be the most recent version. While other versions promise that future releases will still be able to load configurations of this version.

## Configuration

The device simulator runs in your browser. Closing the tab, will drop the configuration. However, you have the following ways to work around this limitation.

### Save a default configuration

You can save one default configuration (per origin), which will be automatically loaded when you open the simulator. The configuration will be stored in the browser's [local storage](https://developer.mozilla.org/en-US/docs/Web/API/Web_Storage_API).

### Copy and paste

You can view and edit the configuration in the YAML editor of the simulator. So you can also copy and paste it, storing a backup of your configuration somewhere you like. It is just YAML.

### Share via URL

You can also create a "share" link from inside the simulator. Which takes the configuration, serializes it, and
base64-encodes it. Adding it to the URL.

When you open the device simulator, it will detect the parameter in the URL, and load this configuration instead of its internally stored, or the default configuration.

## Development

You will need [Rust](https://www.rust-lang.org/) and [Trunk](https://trunkrs.dev/). Once this is installed, you can go
ahead and run:

```shell
trunk serve
```
