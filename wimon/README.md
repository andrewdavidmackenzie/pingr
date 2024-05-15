## Useful commands related to system services

### Linux
`systemctl status mackenzie-serres-pingr.wimon.service` - get a status update (including some logs output)
for the service.

`journalctl` can also be used.

### Macos
Enable it if is disabled
`sudo launchctl enable system/net.mackenzie-service.pingr.wimon`
`sudo launchctl list`

## Power management on macos
Get log of power management events
`pmset -g log | egrep "\b(Sleep|Wake|DarkWake|Start)\s{2,}"`