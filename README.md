# Overview
This program was developed as a replacement for GNU macchanger, because that one seems unmaintained, buggy and somewhat poorly written. The main differences are: macchanger-rust has less features and less bugs while trying to be CLI compatible where easily possible. In general macchanger-rust is a little bit more strict regarding ambigous usage patterns than GNU macchanger and provides (hopefully) better error messages.

# Usage
For building you need a working rust nightly. After you obtained that, run:
```bash
cargo build --release
```

This will create the file `target/release/macchanger`.

To use it, run:
```bash
ip link set eth0 down
macchanger -r eth0
ip link set eth0 up
```

# Project status
We have not yet finished implementing all the macchanger features. Currently only the following options are working:

```
--show
--ending
--random (optionally with --bia)
```

The options `-a` and `-A` will probably never be supported. See `macchanger --help` for more information.

Also reading out the permanent MAC address is currently not implemented. But will be soon.
