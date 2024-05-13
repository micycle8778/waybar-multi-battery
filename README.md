# waybar-multi-battery

Basic custom waybar module for viewing battery info. This module is aware of
several batteries and will sum up the energy of each to display one singular
percentage, unlike the built-in battery module.

Uses UPower via the corresponding `upower` console command, so make sure you
have that installed.

This module outputs in JSON format and uses four CSS classes:

* `charging` for battery charging and fully charged
* `normal` for battery charge above 15%
* `low` for battery charge between (5-15]%
* `critical` for battery charge between [0-5]%

This module currently offers no configuration. The text output is always an 
icon from NerdFonts and the tooltip is always a percentage (and estimated time 
left if available).

Basic waybar config:

```jsonc
    "custom/battery": {
        "exec": "path/to/waybar-multi-battery",
        "return-type": "json",
        "format": "<span>{}</span>",
    },
```

## Building

The program has no build dependencies other than rust crates. Build using Cargo
with:

```
cargo build --release
```

Then make sure to point your waybar config to the binary (default binary path
is `path/to/repo/target/release/waybar-multi-battery`).
