My take on emulating Chip-8 in Rust. Made using winit for window/event loop management, wgpu for graphics/shaders, and rodio for sound. Other dependencies can be found in src/Cargo.toml.

Original Chip-8 inputs controlled with:\
1234 -> 123C\
QWER -> 456D\
ASDF -> 789E\
ZXCV -> A0BF

Space to fast-forward, escape to exit the program, enter to toggle fullscreen.

To run, clone and compile the git repo, then run:\
cargo run --release "path-to-rom"

https://private-user-images.githubusercontent.com/42751478/301918221-419ceff2-f9d1-4c3f-8b7c-a992805ab977.mp4?jwt=eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJnaXRodWIuY29tIiwiYXVkIjoicmF3LmdpdGh1YnVzZXJjb250ZW50LmNvbSIsImtleSI6ImtleTUiLCJleHAiOjE3MDY4OTQwODAsIm5iZiI6MTcwNjg5Mzc4MCwicGF0aCI6Ii80Mjc1MTQ3OC8zMDE5MTgyMjEtNDE5Y2VmZjItZjlkMS00YzNmLThiN2MtYTk5MjgwNWFiOTc3Lm1wND9YLUFtei1BbGdvcml0aG09QVdTNC1ITUFDLVNIQTI1NiZYLUFtei1DcmVkZW50aWFsPUFLSUFWQ09EWUxTQTUzUFFLNFpBJTJGMjAyNDAyMDIlMkZ1cy1lYXN0LTElMkZzMyUyRmF3czRfcmVxdWVzdCZYLUFtei1EYXRlPTIwMjQwMjAyVDE3MDk0MFomWC1BbXotRXhwaXJlcz0zMDAmWC1BbXotU2lnbmF0dXJlPWRjZTg1NjQxZWE2NDA4YjY0YzFmNGRlMzViYTMxYTY5NjE0MjQ5ZjBmNDJmMmE0YTFhMWE3OTk1M2ViNTI5ZDQmWC1BbXotU2lnbmVkSGVhZGVycz1ob3N0JmFjdG9yX2lkPTAma2V5X2lkPTAmcmVwb19pZD0wIn0.KNQTM8pGQKohlX9WZdyZJYe4ZoYfZMOXWHDxxTub9nk

For more information, see:\
https://en.wikipedia.org/wiki/CHIP-8
