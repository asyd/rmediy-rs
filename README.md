# rmediy-rs

A terminal remote control for the **RME ADI-2 DAC / ADI-2 Pro / ADI-2/4 Pro SE**, written in Rust.

RME's official *ADI-2 Remote* app only exists for Windows and macOS. This tool talks to the device over its MIDI/SysEx interface instead, so it runs on Linux.

[![asciicast](https://asciinema.org/a/1267050.svg)](https://asciinema.org/a/1267050)

> Based on the work of [RMEdiy](https://github.com/n00bmax/RMEdiy) by n00bmax (Go, MIT). This is a rewrite in Rust; the protocol follows RME's public MIDI documentation.
> Discussion on the RME forum: <https://forum.rme-audio.de/viewtopic.php?pid=254903#p254903>

## Features

- One tab per output: **Line Out**, **Phones 1/2**, **Phones 3/4** (Pro / Pro SE only)
- Volume (shown in real dB), balance, mute, dim, lock volume, source, reference level
- DA filter, de-emphasis, crossfeed, width, mono, polarity, M/S processing, loopback to USB
- Loudness (enable, bass/treble gain, low volume reference)
- **Parametric EQ**: 5 bands (type, gain, frequency, Q), EQ enable, Bass/Treble, left/right channel (Dual EQ)
- Live **EQ response curve** (computed locally, so it can differ slightly from RME's own graph)
- Device tab: display, clock and lock settings, status information
- A short description of the selected parameter is shown at the bottom of the screen

> **Warning:** this tool sends real commands to the device. Be careful with the volume when headphones or speakers are connected.

## Status

- Tested on an **ADI-2 DAC** only. The Pro and Pro SE parameters follow RME's documentation but are **untested**: feedback is welcome.
- Not done yet: EQ presets (load / store / names) and the remap-keys functions.
- Firmware versions are not exposed over MIDI, so they cannot be displayed (see *SETUP > Options > SW Version* on the device).

## Install

Download the archive from the [releases page](../../releases) (Linux amd64, built on Debian 12) and unpack it. You need the ALSA runtime library:

```sh
sudo apt install libasound2
```

Or build from source (Rust 1.85+, and `libasound2-dev` + `pkg-config` on Debian/Ubuntu):

```sh
cargo build --release
```

## Configuration

The program reads `config.yaml` from the current directory, or the file given as the first argument:

```sh
rmediy-rs                 # uses ./config.yaml
rmediy-rs /path/to/my.yaml
```

```yaml
device:
  id: 0x71            # 0x71 = ADI-2 DAC, 0x72 = ADI-2 Pro, 0x73 = ADI-2/4 Pro SE
  midi_port_in: 1     # index shown by `rmediy-rs --list-ports`
  midi_port_out: 1
sync:
  interval: 10        # seconds between two status requests
```

### Choosing the MIDI ports

`midi_port_in` and `midi_port_out` are **indexes**, not names, and they depend on what is plugged into your machine. List them first:

```console
$ rmediy-rs --list-ports
Entrées MIDI :
  0: Midi Through:Midi Through Port-0 14:0
  1: ADI-2 DAC (59920464):ADI-2 DAC (59920464) Port 1 32:0
Sorties MIDI :
  0: Midi Through:Midi Through Port-0 14:0
  1: ADI-2 DAC (59920464):ADI-2 DAC (59920464) Port 1 32:0
```

Pick the line that contains your ADI-2 (not `Midi Through`, which is a virtual port). The ADI-2 exposes a single bidirectional port, so the input and output indexes are normally the same (here `1` and `1`).

- The indexes can change when you plug or unplug other MIDI devices: run `--list-ports` again if the program cannot connect.
- `amidi -l` (from `alsa-utils`) shows the same device from ALSA's point of view; it appears with the `IO` flag.
- The numeric `id` must match your model, otherwise the device will not answer.
- Phones 3/4 does not exist on the ADI-2 DAC and is hidden for that model.

## Usage

| Key | Action |
|---|---|
| `←` / `→` | Decrease / increase the selected value |
| `↑` / `↓` | Select a parameter |
| `Tab` | Next tab (Line Out, Line EQ, Phones...) |
| `s` | On an EQ tab: switch between the left and right channel (Dual EQ) |
| `r` | Request the full status from the device |
| `q` / `Esc` | Quit |

Frequencies move in proportional steps, and values that the device does not know yet are shown as `—` until it reports them.

## Design notes

The device state is owned by a single async task (tokio) and updated through channels (`mpsc` in, `broadcast` out), so there is no shared mutable map. The original Go version crashed with `concurrent map writes` when MIDI messages arrived concurrently; this design rules that out by construction.

## License

MIT, see [LICENSE](LICENSE). Includes the copyright of the original RMEdiy project.
The device protocol is documented by RME ("MIDI-Protocol for RME ADI-2 DAC, ADI-2 Pro and ADI-2/4 Pro SE").
