# Drgctl

A Command line tool for managing apps and devices in a drogue cloud instance. 

## Installation 
Todo (download from releases)

## Usage

### Resources 

`drgctl` interacts with resources existing in drogue-cloud, currently `apps` and  `devices` operations are supported. 
The following operations are handled : 
* create
* delete
* edit
* get

### Apps operation

```
# Create an app 
drgctl -url https://drogue-cloud-registry-endpoint create app <appId>
# adding data
drgctl create app <appId> -d `{"foo":"bar"}`

#edit an app 
drgctl edit app <appId>

# Delete an app 
drgctl delete app <appId>
```

### Apps operation

```
#Create a device
drgctl -url https://drogue-cloud-registry-endpoint create device <deviceId> --app <appId>
# adding data
drgctl create device <deviceId> --app <appId> -d `{"foo":"bar"}`

#edit a device data 
drgctl edit device <deviceId> --app <appId>

# Delete a device 
drgctl delete device <deviceId> --app <appId>
```

## Using a configuration fie

`drgctl` will load cluster settings from a configuration file. The `DRGCFG` environment variable can point to a config file location.
The default config file location is `$HOME/.drgconfig.json`. This default value will be used if the environment variable is not set. 
This location can be overriden with the `--config` argument : 
```
drgctl --config path/to/config create device <deviceId> --app <appId>
```

Config file minimal example: 
``` 
{
    "drogue_cloud_url": "https://registry.sandbox.drogue.cloud",
}
``` 

## Roadmap

In no particular order here are the following things that we would like to add to `drgctl` :
 * List resources
 * Trust anchors support
 * Json patch operations
 * Other platforms binaries : MacOS and windows.