# NES

[![CircleCI](https://circleci.com/gh/delta62/nes/tree/main.svg?style=svg&circle-token=d116670ebb1019abeff56776e5d2f4835e66b783)](https://circleci.com/gh/delta62/nes/tree/main)

An NES emulation library

This is a Cargo workspace with several members:

- **debugger** is a debugging UI for the emulation logic
- **imgui-glfw** is a renderer for imgui, needed for the debugger
- **nes** is the core NES emulator, containing only emulation logic
- **pcm** is a low-level PCM output library (for playing sound)
- **scap** is a **s**creen **cap**ture library for recording output of the NES
