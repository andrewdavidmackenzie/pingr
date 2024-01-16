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
npx wrangler build
```

### Running collectr locally
```commandline
npx wrangler dev
```

If you want to run a local `wimon` against this running version of `collectr`
then you will need to edit the `wimon.toml` file in the wimon directory
something like this:
```commandline
...
base_url = "http://localhost:8787"
#base_url = "https://collectr.mackenzie-serres.workers.dev"
...
```

### Deploying collectr to cloudflare's network
```commandline
npx wrangler deploy
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