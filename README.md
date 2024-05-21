![](icon.png)

# pingr

Pingr is a project to help monitor remote network connections by deploying a small monitoring
program at the remote site (could be on an embedded device, a raspberry pi or a
mac or linux PC) connected via the connection (Wifi or Ethernet) you want to monitor.

It regularly will send reports to a web service, and if a report is missed, then
the service will mark the device as "offline" and the connection as offline.

Connections status can then be monitored via an app and the user alerted
that something is wrong with their connection and some maintenance
action maybe needed.

It can also measure network strength and quality and send qualitative metrics
on the connection.

## collectr

`collectr` is a cloud service that receives the reports from monitoring programs/devices.

The current implementation is a cloudflare worker in rust, using Durable Objects.

Status is saved to the cloudflare key-value store and maybe queried to see
device and connection state.

I'm pending approval for cloudflare's Beta Pub/Sub service to be able to use that both
to send reports, as well as subscribe to status change events for devices and connections.

The following sections on developing `collectr` require that you install cloudflare's development
tools, including `wrangler`

### Building collectr

You can build `collectr` using

```commandline
wrangler build
```

### Running collectr locally

```commandline
wrangler dev
```

If you want to run a local `wimon` against this running version of `collectr`
then you will need to edit the `wimon.toml` file in the wimon directory
something like this:

```toml
#<removed text>
base_url = "http://localhost:8787"
#base_url = "https://collectr.mackenzie-serres.workers.dev"
#<removed text>
```

### Deploying collectr to cloudflare's network

```commandline
wrangler deploy
```

## wimon

`wimon` (can be pronounced with a Scottish or Jamaican accept :-) ) is a small "wifi monitoring"
command line application, which when run on a remove device will send regular reports to
`collector`.

### Building wimon

You can build `wimon` using

```commandline
cd wimon
cargo build
```

#### Building wimon on Raspberry Pi

libssl headers are required, install with:

```commandline
sudo apt-get install libssl-dev
```

#### Running wimon

You can run the wimon binary with the standard options using:

```commandline
cd wimon
cargo run
```

#### Wimon Config

Currently wimon looks for a config file called `wimon.toml` in the directory where it is executed, and then it searches
all the parent directories between that directory and root looking for the same config file, stopping as soon as one
is found. Then it loads the config from there. This may change in the future.

#### Installing wimon as a service (Macos, Linux, Window)

To install `wimon` as a background service (and start it immediately) that is also re-started at boot,
execute it with the "install" command:

```commandline
sudo cargo run -- install
```

#### Uninstalling wimon as a service (Macos, Linux, Window)

To remove the installed `wimon` background service (after stopping it first) execute it with the "uninstall" command:

```commandline
sudo cargo run -- uninstall
```

#### Check the status of service on linux

You can check the current status and get last output using:

```commandline
systemctl status mackenzie-serres-pingr.wimon.service
```

which should return something like:

```● mackenzie-serres-pingr.wimon.service - mackenzie-serres-pingr.wimon
     Loaded: loaded (/etc/systemd/system/mackenzie-serres-pingr.wimon.service; enabled; vendor preset: enabled)
     Active: active (running) since Fri 2024-01-19 12:28:13 CET; 2min 0s ago
   Main PID: 2619 (wimon)
      Tasks: 2 (limit: 414)
        CPU: 618ms
     CGroup: /system.slice/mackenzie-serres-pingr.wimon.service
             └─2619 /home/andrew/workspace/pingr/target/debug/wimon

Jan 19 12:28:13 pizerow0 systemd[1]: Started mackenzie-serres-pingr.wimon.
Jan 19 12:28:13 pizerow0 wimon[2619]: Config file loaded from: "/home/andrew/workspace/pingr/wimon.toml"
Jan 19 12:28:13 pizerow0 wimon[2619]: Monitor: Connection
Jan 19 12:28:15 pizerow0 wimon[2619]: Sent OnGoing report to: collectr.mackenzie-serres.workers.dev
Jan 19 12:28:15 pizerow0 wimon[2619]: Response: Device ID: c6426011b76adc13c41ffd737c0a07b2495f59a2bc94f725d26>
Jan 19 12:29:17 pizerow0 wimon[2619]: Sent OnGoing report to: collectr.mackenzie-serres.workers.dev
Jan 19 12:29:17 pizerow0 wimon[2619]: Response: Device ID: c6426011b76adc13c41ffd737c0a07b2495f59a2bc94f725d26>
```

#### Testing wimon

Test wimon locally using

```commandline
cargo test
```

### Supported platforms

Currently `wimon` has been tested to run on:

- macos
- linux (ubuntu)
- raspberry pi 4 (Pi400 in fact) with Raspberry Pi OS
- raspberry pi zero (W - with wifi) with Raspberry Pi OS