Issues and PR are welcome !!

## Running integration tests

GitHub actions runs integrations tests for drg, against the [drogue-cloud sandbox](https://sandbox.drogue.cloud).
They are not ran in pull requests to keep the sandbox data safe.

However, you can run them locally to check if you haven't broken things.
In order to do that you need to have the following `.env` at the root of the repo : 
```dotenv
DROGUE_SANDBOX_ACCESS_KEY=<your_complete_access_key>
DROGUE_SANDBOX_KEY_PREFIX=<your_access_key_prefix>
DROGUE_SANDBOX_USERNAME=<your_username
DROGUE_SANDBOX_URL=https://api.sandbox.drogue.cloud/
```

To create an access key you can use the console (In the left menu see API>Access tokens).
Then you can run the tests with `cargo test`.