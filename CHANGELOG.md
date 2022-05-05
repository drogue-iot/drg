# Version 0.9.0

## New features
- Add a json output : `-o json`
- Add a `-o wide` option when listing devices to show more information. Thanks to [lulf](https://github.com/lulf) !
- Drg now uses `drogue-client` under the hood.

## Misc. changes
- Big internal refactoring improving code maintainability.
- Add integration tests

# Version 0.8.1

## New features
- Reworked the CLI flow : general improvement in usability, better help messages.
- drg can now set labels using `drg set labels` subcommand.
- drg stream can now filter the stream to only display messages coming from a given device.


## Misc. changes
- When creating or updating an application or a device with a full spec file, the application name or device Id will be red from the file, without needing to input it.
- When creating a token, a description can be optionally provided.

## Dependencies
- Updated to clap v3
- updated to oauth v4

# Version 0.8.0

## New features
- Added an `admin` subcommand to manage application members, transfer application ownership and manage access tokens.
- Added optional -n <count> parameter for the `stream` command to stream a fixed number of messages.

## Bug fixes
- Add URL encoding for user supplied values. 

## Misc. changes
- Add aliases for `drg stream` : `consume`, `subscribe` are now valid aliases. 

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
 
## Bug fixes

## Misc. changes
 - Improved debug messages related to the open ID authentication flow.
 - When using `edit`, drg won't send anything to the server if there are no changes.
 - Automated builds 
 - Add an "ignore-missing" flag to ignore 404 error when deleting a resource.
 

## Deprecations
 - `drg token` is removed. Please use `drg whoami --token`
 - `--data` argument is deprecated. Use `--spec` instead.
 
