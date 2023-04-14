# Installation instructions

## Systemd

If not already done, clone the repo and enter the directory:

```
git clone https://github.com/ccicconetti/etsi-mec-qkd.git
cd etsi-mec-qkd
```

Create an example `application_list.json` file with:

```
cargo test test_message_application_list_to_json -- --ignored
```

Build, and install the `lcmp` executable as a service on Linux with [systemd](https://wiki.archlinux.org/title/systemd):

```
cargo build -r
mkdir /opt/lcmp
sudo cp application_list.json /opt/lcmp
sudo cp target/release/lcmp /opt/lcmp
sudo cp systemd/lcmp.sh /opt/lcmp
sudo cp systemd/lcmp.service /etc/systemd/system
sudo systemctl start lcmp
sudo systemctl enable lcmp
```

### Status check and monitoring

You can check the status of the service with:

```
systemctl status lcmp
```

and monitor the live logs with:

```
journalctl -u lcmp -f
```

### Customization

1. Substitute the example `application_list.json` file with the real one containing the specs of your applications
2. Substitute `URI` in `/opt/lcmp/lcmp.sh` with the real URI that should be returned when creating new application contexts from a device app
3. Reload the service with `systemctl restart lcmp`