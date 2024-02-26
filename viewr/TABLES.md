# Tables description
This file describes the tables in the KV store being used by collectr and viewr.

`collectr` consists of:
- A `DurableObject` that tracks the status of a reporting device. It stores the DO state in the DO's own state
storage and it does not interact with the KV store. When a device changes state, a message is sent to the
"state-changes" queue (`binding = "STATE_CHANGES"`). 
- A `Worker` that:
  - Receives monitoring reports from devices as http requests. It passes these directly to the `DO`
that tracks the device's state.
  - Processes `StateChange` events from the "stats-changes" queue:
    - Updating the device's status in the `DEVICE_STATUS` KV Namespace
    - Creates a DeviceID to Connection Mapping in the `DEVICE_ID_CONNECTION_MAPPING` KV Namespace if one does not 
      already exist.
    - Updates the device's status in the `CONNECTION_DEVICE_STATUS` KV Namespace
    - Creates an entry for the device in the `DEVICE_DETAILS` KV Namespace, with default contents, if one does not 
      already exist. This is to facilitate later editing of the details for new devices being added.

They are referred to using the binding name (see wrangler.toml). Generally, in the rust source
files there is a constant defined that matches that name, and that is what is used in the code.

| Name                         | Visibility | Key Structure                   | Contents            |
|------------------------------|------------|---------------------------------|---------------------|
| DEVICE_ACCOUNT_MAPPING       | Device     | DeviceID                        | AccountId           |
| DEVICE_ID_CONNECTION_MAPPING | Account    | DeviceID                        | Connection          |
| DEVICE_ID_CONNECTION_MAPPING | Account    | DeviceID                        | Connection          |
| DEVICE_STATUS                | Account    | DeviceID                        | DeviceStatus        |
| CONNECTION_DEVICE_STATUS     | Account    | ConnectionDescription::DeviceID | DeviceStatus        |
| CONNECTION_LIST              | Account    | not implemented yet             | not implemented yet |
| DEVICE_DETAILS               | Account    | DeviceID                        | DeviceDetails       |

## Key Structure
Describes how the key is formed. KV stores support listing keys that match a prefix, and this is used in some
of the tables to enable particular functionality, hence the components of the key and their order is critical.

## Visibility
Visibility of the table's contents to components, accounts, users etc. 

A valid visibility should be passed to all methods to read or write to/from the given table.
- Visibility::Device - requires a valid DeviceId
- Visibility::Account - requires a valid AccountId

- Will also need a userid to accountid mapping, so that when a user logs in the accountid can be found and then 
used in all subsequent requests to these tables? Or could account id be some meta-data associated with user so that
we get it as soon as they are logged in?

TODO
modify the relevant api methods, so that accept the visibility parameter (which will be a String to be used as 
the prefix)

TODO method to associate a device to a specific account, creating an entry into the Device_Account_Mapping table.

## Contents 
Describes what is in the `value` part of this key-value pair.

## Descriptions
### `DEVICE_ACCOUNT_MAPPING`
This is used to fetch the `AccountId` associated with a device (via `DeviceId`), so that the worked when it
receives events on a device, can fetch the `AccountId` so it can use that in the update of all subsequent
tables.

### `DEVICE_ID_CONNECTION_MAPPING`

### `DEVICE_STATUS`

### `CONNECTION_DEVICE_STATUS`

### `CONNECTION_LIST`
Not implemented yet.

### `DEVICE_DETAILS`
Used to contain details describing a device, entered by an admin via the UI, not as reported by the device.

## Types 
### `DeviceID`
The unique ID of the DurableObject that represents a Device, as a String.

e.g. `5082eee0ff53ce01450a19252c80c816905c941c1ee56bfa319494b908409d1f

### `Connection`
Serialization of `Connection` enum type to a String.

e.g. `ssid=MOVISTAR_8A9E`

### `DeviceStatus`
Serialization of `DeviceStatus` type to a String.

e.g. `Reporting`, `Offline` or `Stopped`

### `DeviceStatus`
Serialization of `DeviceDetails` type to JSON.

e.g. `{"friendly_name" : "PiZeroW0"}`

### `AccountId`
Serialization of `AccountId` to String. Account Ids should be large and very hard to guess.

e.g. hdsakhsjklfdsjñfjks