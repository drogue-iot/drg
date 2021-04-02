# drg : Drogue cloud command line tool

A Command line tool for managing apps and devices in a drogue cloud instance. 

# Installation 

Download the latest release from the github release page and place it somewhere in your `$PATH`.
Or build from source via crates.io: 
```
cargo install drg
```


# Usage

## Log in to a drogue cloud instance

In order to use `drg` to manage resources in drogue cloud you first need to authenticate : 
```
drg login https://drogue-cloud-registry-endpoint
```
Then follow the steps to authenticate. drg will generate a config file to save your configuration.

## Managing resources 

`drg` interacts with resources existing in drogue-cloud, currently `apps` and  `devices` operations are supported. 
The following operations are handled :
* create
* delete
* edit
* get

## Apps operations

```
# Create an app 
drg create app <appId>
# adding data
drg create app <appId> -d `{"foo":"bar"}`

# edit an app - this will open an editor. 
drg edit app <appId>

# update an app providing the data
drg update app <appId> -f </path/to/json>

# Delete an app 
drg delete app <appId>
```

## Devices operations

```
# Create a device
drg create device <deviceId> --app <appId>    # --app and -a are interchangeable
# Add some data
drg create device <deviceId> -a <appId> -d `{"foo":"bar"}`

# Edit a device data - this will open an editor
drg edit device <deviceId> --app <appId>

# update a device providing the data
drg update device <deviceId> -a <appId> -f </path/to/json>

# Delete a device 
drg delete device <deviceId> - <appId>
```

## Configuration fie

`drg` will load cluster settings from a configuration file. The `DRGCFG` environment variable can point to a config file location.
The default config file location is `$HOME/.config/drg_config.json`. This default value will be used if the environment variable is not set. 
This location can be overriden with the `--config` argument : 
```
drg --config path/to/config create device <deviceId> --app <appId>
```

To get a working config file, run see [login to a drogue cloud instance](#Log-in-to-a-drogue-cloud-instance)

# Roadmap

In no particular order here are the following things that we would like to add to `drg` :
 * List resources
 * Trust anchors support
 * Json patch operations
 * Other platforms binaries : MacOS and windows.