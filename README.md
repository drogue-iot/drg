[![Release (latest SemVer)](https://img.shields.io/github/v/tag/drogue-iot/drg?sort=semver)](https://github.com/drogue-iot/drg/releases)
[![Build](https://github.com/drogue-iot/drg/actions/workflows/build.yaml/badge.svg?branch=main)](https://github.com/drogue-iot/drg/actions/workflows/build.yaml)
[![Matrix](https://img.shields.io/matrix/drogue-iot:matrix.org)](https://matrix.to/#/#drogue-iot:matrix.org)

# drg : Drogue cloud command line tool

A Command line tool for managing apps and devices in a drogue cloud instance. 

# Installation 

## Install from sources 

Via crates.io:

    cargo install drg

## Download a release

Download the latest release from the [github release page](https://github.com/drogue-iot/drg/releases) and place it somewhere in your `$PATH`.

Note: Debian users must install the `libssl1.1` package.

## Homebrew

    brew tap drogue-iot/drg
    brew install drg

## Fedora 

Starting with Fedora 34, you can install `drg` directly from the Fedora repositories:

    sudo dnf install drg

## Snap

    sudo snap install drogue-cli
    sudo snap alias drogue-cli drg

# Usage

## Log in to a drogue cloud instance

In order to use `drg` to manage resources in drogue cloud you first need to authenticate : 
    
    drg login https://drogue-cloud-api-endpoint

Then follow the steps to authenticate. drg will generate a config file to save your configuration.

You can also use a refresh token to authenticate, suitable when the browser can't be accessed:
    
    drg login https://drogue-cloud-api-endpoint --token <refresh_token>


## Managing resources 

`drg` interacts with resources existing in drogue-cloud, currently `apps` and  `devices` operations are supported. 
The following operations are handled :
* create
* delete
* edit
* get
* list

###  Create resources

    # Create an app 
    drg create app <appId>
    # adding data
    drg create app <appId> -d `{"foo":"bar"}`
    
    # Create a device
    drg create device <deviceId> --app <appId>    # --app and -a are interchangeable
    # Add some data
    drg create device <deviceId> -a <appId> -d `{"foo":"bar"}`
    
### Read resources

    # Read an app
    drg get appp <appId>
    # Get a list of apps
    drg get apps
    
    # Read a device
    drg get device <deviceId> --app <appId>
    # Get a list of devices
    drg get devices --app <appId>
    
Note: `list` support adding labels for filtering results:

          # Get a list of devices (here all 3 labels will be applied.
          drg get apps -l key=value,foo=bar --label fiz=buz
    
### Edit and delete resources
    
    # edit an app - this will open an editor. 
    drg edit app <appId>
    
    # update an app providing the data
    drg edit app <appId> -f </path/to/json>
    
    # Edit a device data - this will open an editor
    drg edit device <deviceId> --app <appId>
    
    # update a device providing the data
    drg edit device <deviceId> -a <appId> -f </path/to/json>
    
    # Delete an app 
    drg delete app <appId>
    
    # Delete a device 
    drg delete device <deviceId> - <appId>

## Configuration file

`drg` will load cluster settings from the default context of a configuration file. The `DRGCFG` environment variable can point to a config file location.
The default config file location is `$HOME/.config/drg_config.yaml`. This default value will be used if the environment variable is not set. 
This location can be overriden with the `--config` argument : 
   
    drg --config path/to/config create device <deviceId> --app <appId>

To get a working config file, run see [login to a drogue cloud instance](#Log-in-to-a-drogue-cloud-instance)

### Context management

A valid configuration can contain multiple context allowing you to switch between cluster easily. 
To create a new context simply log into a cluster with `drg login` : [login to a drogue cloud instance](#Log-in-to-a-drogue-cloud-instance)
If it's the first context created for this configuration file it will be set as active by default. 

To update the active context for a config file : 
    
    drg context set-active <contextId>

Here are some other commads available to manage contexts :

    drg context show #will display the whole config file. 
    drg context list
    drg context set-default-app <appId> #will use active context
    drg context set-default-app <appId> --context <anotherContextId>
    drg context delete <contextId> 
    drg context rename <contextId> <newContextId>

context and app can be set with environment variables : `DRG_CONTEXT` and `DRG_APP`.

### Trust-anchor management

Drogue cloud has support for authentication of devices using x509 certificates.
To enable that we need to create a root CA and add it to the application object.

    drg trust create --app <appId> --keyout <filename>

Once Trust-anchor is set, we can use it to sign device certificates, for example:

    drg trust add --app <appId> --device <deviceId> --CAkey <app-private-key> --out <filename> --keyout <filename>

# Roadmap

In no particular order here are the following things that we would like to add to `drg` :
 * Trust anchors support
 * Json patch operations