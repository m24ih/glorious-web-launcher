# packaging

## udev rule

The service opens the vendor HID collection directly (`/dev/hidraw*`), so the
logged-in user needs read/write access. Install the rule once:

```sh
sudo cp packaging/99-glorious.rules /etc/udev/rules.d/
sudo udevadm control --reload-rules
sudo udevadm trigger
```

Replug the mouse or its dongle afterwards. `TAG+="uaccess"` grants access to the
active local session without world-writable device nodes.
