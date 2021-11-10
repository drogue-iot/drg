# Version 0.7.1

## New features

## Bug fixes

## Misc. changes

## Deprecations


# Version 0.7.0

## New features
 - Added a `stream` subcommand to tap in the websocket integration service and stream events
 - Added a `trust` subcommand to manage certificates and trust anchors for devices and apps. 
 - Added a `set` operation to easily add credentials, gateway or alias to a device. 
 - Devices and apps can now be listed if not ID is specified :  `drg get apps` will list existing apps. 
 Plural and singular forms of a resource can be used interchangeably.
 - Endpoints information in `drg whoami -e`. It's also possible to specify a service name to get only the url.
 - Added a `cmd` subcommand to issue commands for devices using the command endpoint.
 - Added an `admin` subcommand to manage application members, transfer application ownership and manage access tokens.
 
## Bug fixes

## Misc. changes
 - Improved debug messages related to the open ID authentication flow.
 - When using `edit`, drg won't send anything to the server if there are no changes.
 - Automated builds 
 - Add an "ignore-missing" flag to ignore 404 error when deleting a resource.
 

## Deprecations
 - `drg token` is removed. Please use `drg whoami --token`
 - `--data` argument is deprecated. Use `--spec` instead.
 