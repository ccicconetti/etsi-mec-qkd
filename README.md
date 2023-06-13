# etsi-mec-qkd

Documents and software about the integration of ETSI MEC and ETSI QKD. Activities started within the project PON QUANCOM funded by the Italian Ministry of University and Research

## ETSI MEC LCMP

A partial implementation of the ETSI MEC Life Cycle Management Proxy is included.

### Compilation

Prerequisites:

- rust, with minimum supported Rust version (MSRV) of 1.56, install following the official [instructions](https://www.rust-lang.org/tools/install)

Clone the git repository:

```
git clone https://github.com/ccicconetti/etsi-mec-qkd.git
cd etsi-mec-qkd
```

Optionally build in debug mode and execute the unit tests:

```
cargo build
cargo test
```

Build the release version:

```
cargo build -r
```

The executable can be found as `target/release/lcmp`.

Without parameters (see command-line options with `-h`) it will look for a file `application_list.json` in the current directory, which contains the list of meApps to be made available to the device apps.

### Installation

See the [dedicated instructions](systemd/README.md).

### Execution example

#### GET application list

Create an example `application_list.json` file with:

```
cargo test test_message_application_list_to_json -- --ignored
```

Then run in one shell:

```
target/release/lcmp
```

and in another:

```
curl -H "Content-type: application/json" http://localhost:8080/dev_app/v1/app_list
```

#### POST/PUT/DELETE AppContext

Create example `application_list.json` and `app_context.json` files with:

```
cargo test test_message_application_list_to_json -- --ignored
cargo test test_message_application_app_context -- --ignored
```

Then run in one shell:

```
target/release/lcmp
```

and in another:

```
curl -d@- -X POST -H "Content-type: application/json" http://localhost:8080/dev_app/v1/app_contexts < app_context.json | python -m json.tool > new_context.json
```

By changing the `callbackReference` in `new_context.json` you can update the context with:

```
CONTEXTID=$(grep contextId new_context.json   | cut -f 4 -d '"')
curl -d@- -X PUT -H "Content-type: application/json" http://localhost:8080/dev_app/v1/app_contexts/$CONTEXTID < new_context.json
```

As can be seen with:

```
curl -X GET http://localhost:8080/dev_app/v1/app_contexts/$CONTEXTID
```

You can also check the list of active contexts with the following _non-standard_ command:

```
curl -X GET http://localhost:8080/dev_app/v1/app_contexts
```

Finally, the context can be deleted with:

```
curl -X DELETE http://localhost:8080/dev_app/v1/app_contexts/$CONTEXTID
```
