# Version 0.7.0

## New features
 - Added a `stream` subcommand to tap in the websocket integration service and stream events
 - Added a `trust` subcommand to manage certificates and trust anchors for devices and apps. 
 - Added a `set` operation to easily add credentials, gateway or alias to a device. 
 - Devices and apps can now be listed if not ID is specified :  `drg get apps` will list existing apps. 
 Plural and singular forms of a resource can be used interchangeably.
 - Endpoints information in whoami -e. It's also possible to specify a service name to get only the url.
 - Added a `cmd` subcommand to issue commands for devices using the command endpoint. 
 
## Bug fixes

## Misc. changes
 - Improved debug messages related to the open ID authentication flow.
 - When using `edit`, drg won't send anything to the server id there are no changes.
 - Automated builds and packaging for fedora
 

## Deprecations
 - `drg token` is removed. Please use `drg whoami --token`
 
