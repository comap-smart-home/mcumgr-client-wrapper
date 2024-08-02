# mcumgr-client-wrapper

Python bindings for the rust library [mcumgr-client](https://github.com/vouch-opensource/mcumgr-client).
This allows sending MCUmgr commands to a device connected to a serial port from Python.

# How to use

```
import mcumgr_client as mcu

s = mcu.SerialSession(device='/dev/ttyUSB0', baudrate=576000)
# Get a list of dicts containing image properties 
l = s.list()
print(l)

# Upload image to device
s.upload('/path/to/image/bin')

# Set the pending flag on the uploaded image
img_hash = [34, 33, 245]
s.test(img_hash)

# Delete image
s.delete(slot=1)

# Reset the device
s.reset()
```

see `help(mcumgr_client)` for more