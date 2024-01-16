# Beacon: a language for generating patterns for an LED strip

This is a toy that I banged together in Rust. It generates ripply color
patterns that you could use to control a strip of individually-addressable
LEDs.

There is no code to interface *with* an LED strip. For that, you need
a different project. I suspect you should start with [this one][smart-leds],
but I haven't really dug into it yet.

[smart-leds]: https://github.com/smart-leds-rs/smart-leds

All *this* project does is write RGB values into an array of floats,
and then display them as a strip of colors in a window (using SDL2).
The point is that the colors are controlled by a script, and it's very
easy to write and iterate scripts. See the scripts directory for examples.

## Try it out

```
cargo run scripts/bustle.pab
```

Or try one of the other scripts. I'm fond of `clouds.pab` and `portal.pab`.

By default this simulates a strip of 160 LEDs. For a higher density, try

```
cargo run scripts/portal.pab --size 320
```
