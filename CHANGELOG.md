# Version 0.11

## New features
- Added a `drg apply` subcommand. It's tailored to feel like the `kubectl apply` we are used to.
- Added support for PSK credentials (see drogue-cloud 0.11 release.

## Misc. changes
- updated CI runners

## Dependencies
- Updated to colored_json 3
- updated to drogue-client 0.11

# Version 0.10.2

## New features
- New subcommand : `apply` allows to create or update devices and application from json or yaml. Works like `kubectl apply`.
- Added support for channel filtering in `drg stream` with `--channel` option. 

## Misc. changes
- Added missing JSON output for `version`

# Version 0.10.1

## Bug fixes
- Fixed an issue where `drg config default-app` and `default-algo` were not saving the config changes

## Misc. changes
- add plural aliases for member subcommands: 

# Version 0.10.0

## New features
- Added an interactive / terminal mode : start it with `drg --interactive`
- Added an `--insecure` flag for `drg stream` allowing to connect to servers using self-signed certificates (e.g. drogue-server)

## Misc. changes
- Added missing JSON output for `login`, `config` and `whoami`
- Added support for refreshing the auth token in `drg stream` to keep alive the connection

## Dependencies
- Updated to tungstenite 0.17.2
- updated to tiny_http 0.8.0
- updated to drogue-client 0.10.1

# Version 0.9.0

## New features
- Reworked the CLI flow : general improvement in usability, better help messages.
- drg can now set labels using `drg label` subcommand : `drg label device myDevice aLabel bar=baz` or `drg label app myApp key=val someOtherLabel`
- drg stream can now filter the stream to only display messages coming from a given device.
- Add a json output : `-o json`
- Add a `-o wide` option when listing devices to show more information. Thanks to [lulf](https://github.com/lulf) !
- Drg now uses `drogue-client` under the hood.
- Add a "--active" flag to `drg config show` to only show the current active context.

## Misc. changes
- Swap the position of passwords arguments for `drg set password` to make it more aligned with how the data hierarchy. 
- When creating or updating an application or a device with a full spec file, the application name or device Id will be red from the file, without needing to input it.
- When creating a token, a description can be optionally provided.
- Big internal refactoring improving code maintainability.
- Add integration tests

## Bug fixes
- Prevents renaming a context with a name that already exist in the config file.

## Dependencies
- Updated to clap 3.0
- updated to oauth 4.1.0
- updated to tabular 0.2.0

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
 
