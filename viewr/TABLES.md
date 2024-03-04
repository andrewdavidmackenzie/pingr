# Tables description
This file describes the tables in the KV store being used by collectr (worker only) and viewr (UI).

`collectr` consists of:
- A `DurableObject` that tracks the status of a reporting device. It stores the DO state in the DO's own state
storage and it does not interact with the KV store. When a device changes state, a message is sent to the
"state-changes" queue (`binding = "STATE_CHANGES"`). 
- A `Worker` that:
  - Receives monitoring reports from devices as http requests. It passes these directly to the `DO`
that tracks the device's state.
  - Processes `StateChange` events from the "stats-changes" queue:
    - Updating the device's status in the `DEVICE_STATUS` KV Namespace
    - Updates the device's status in the `CONNECTION_DEVICE_STATUS` KV Namespace
    - Creates an entry for the device in the `DEVICE_DETAILS` KV Namespace, with default contents, if one does not 
      already exist. This is to facilitate later editing of the details for new devices being added.

They are referred to using the binding name (see wrangler.toml). Generally, in the rust source
files there is a constant defined that matches that name, and that is what is used in the code.

| Table Name               | Visibility | Key Structure                   | Contents            |
|--------------------------|------------|---------------------------------|---------------------|
| DEVICE_ACCOUNT_MAPPING   | Global     | DeviceID                        | AccountId           |
| DEVICE_STATUS            | Account    | DeviceID                        | StateChange         |
| CONNECTION_DEVICE_STATUS | Account    | ConnectionDescription::DeviceID | StateChange         |
| DEVICE_DETAILS           | Account    | DeviceID                        | DeviceDetails       |

## Visibility
Visibility restricts the code's ability to work with data in a Table, for the purposes of security and 
multi-tenancy.

### `Global` Visibility
Anyone able to form a valid key for the table can work with it.

### `Account` Visibility
To be able to work with data in tables with `Account` visibility, code must have a valid `AccountId`, plus be able to 
form valid keys.
It will only be possible to work with (Create, Read, Update, Delete) Key-Value pairs that belong to the account 
associated with the `AccountId`.

## Key Structure
Describes how the key is formed. KV stores support listing keys that match a prefix, and this is used in some
of the tables to enable particular functionality, hence the components of the key and their order is critical.

## Contents 
Describes what is in the `value` part of this key-value pair.

## Table Descriptions
### `DEVICE_ACCOUNT_MAPPING`
This is used to fetch the `AccountId` associated with a device (via `DeviceId`), so that the worked when it
receives events on a device, can fetch the `AccountId` so it can use that in the update of all subsequent
tables.

### `DEVICE_STATUS`
For each device, this stores the status it is in, as tracked by it's Durable Object.
The Value stored is a `StateChange` struct, which as the state of the device, an optional connection and the
timestamp when the change to that state occurred.

### `CONNECTION_DEVICE_STATUS`
Allows us to form a list of Connections with each of the devices reporting against it, for use in the UI.
The Value stored is a `StateChange` struct, which as the state of the device, an optional connection and the
timestamp when the change to that state occurred.

### `DEVICE_DETAILS`
Used to contain details describing a device, entered by an admin via the UI, not as reported by the device.

## Data Types 
### `DeviceID`
The unique ID of the DurableObject that represents a Device, as a String.

e.g. `5082eee0ff53ce01450a19252c80c816905c941c1ee56bfa319494b908409d1f`

### `Connection`
Serialization of `Connection` enum type to a String.

e.g. `ssid=MOVISTAR_8A9E`

### `DeviceStatus`
Serialization of `DeviceStatus` type to a String.

e.g. `Reporting`, `Offline` or `Stopped`

### `StateChange`
Serialization of `StageChange` struct to JSON.

e.g. `{"friendly_name" : "PiZeroW0"}`

### `AccountId`
Serialization of `AccountId` to String. Account Ids should be large and very hard to guess.
TODO might rename to GroupId....

e.g. hdsakhsjklfdsjñfjks