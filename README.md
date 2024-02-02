My take on emulating Chip-8 in Rust. Made using winit for window/event loop management, wgpu for graphics/shaders, and rodio for sound. Other dependencies can be found in src/Cargo.toml.

Original Chip-8 inputs controlled with:
1234 -> 123C\
QWER -> 456D\
ASDF -> 789E\
ZXCV -> A0BF\

Space to fast-forward, escape to exit the program, enter to toggle fullscreen.

For more information, see:\
https://en.wikipedia.org/wiki/CHIP-8
