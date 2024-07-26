#!/usr/bin/env python3

import sys
import mcumgr_client as mcu

s = mcu.SerialSession(sys.argv[1], 576000)
d = s.list()
print(d)

s.upload(sys.argv[2])

d = s.list()
print(d)

s.reset()
