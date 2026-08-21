# Rocci Blocks

Local solo preview. Published tutorial: https://rocci.dev/examples/blocks/

```sh
cargo run -q -p rocci-cli -- inspect --ast examples/rocci/custom/blocks/Blocks.rocci
cargo run -q -p rocci-cli -- view examples/rocci/custom/blocks/Blocks.rocci --component PlayPage
cargo run -q -p rocci-cli -- run examples/rocci/custom/blocks/Blocks.rocci
```

Keyboard: arrows / WASD move, Up/X/W rotate CW, Z CCW, Space hard drop.
Touch: on-screen pad. Gamepad axes and face buttons are also read.
