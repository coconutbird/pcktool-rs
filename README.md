# pcktool-rs

A Rust tool for inspecting and extracting Wwise `.pck` and `.bnk` audio packages.

## Features

- Parse PCK containers (language maps, sound bank / streaming / external file LUTs)
- Parse BNK sound banks (BKHD, DIDX/DATA embedded media, HIRC)
- **Big-endian support** — auto-detects byte order (e.g. `Sounds_china.pck`)
- Extract all audio sorted by language into a clean directory tree
- `no_std` + `alloc` library core (`pcktool`), usable in embedded contexts
- Zero-copy parsing — file entries borrow directly from the memory-mapped source

## Workspace

| Crate         | Description                                    |
| ------------- | ---------------------------------------------- |
| `pcktool`     | `no_std` library — PCK/BNK parsing and writing |
| `pcktool-cli` | CLI binary — `info`, `list`, `dump` commands   |

## Usage

```
pcktool <COMMAND>

Commands:
  info    Show information about a PCK or BNK file
  list    List entries in a PCK file (banks, streaming, external)
  dump    Extract audio files from a PCK or BNK file
```

### info

Display summary information about a `.pck` or `.bnk` file.

```sh
pcktool info Sounds.pck
```

```
PCK File Info
─────────────────────────────────
Languages:        7
  [0] sfx
  [1] english(us)
  [2] french(france)
  [3] german
  [4] italian
  [5] japanese
  [6] russian
Sound Banks:      433
Streaming Files:  42385
External Files:   0
```

### list

List entries in a PCK file. Defaults to sound banks.

```sh
pcktool list Sounds.pck            # list sound banks
pcktool list -s Sounds.pck         # list streaming files
pcktool list -e Sounds.pck         # list external files
```

### dump

Extract all files from a PCK or BNK, organized by language.

```sh
pcktool dump Sounds.pck -o output/
```

Produces a directory tree like:

```
output/
├── SFX/
│   ├── banks/
│   │   ├── 06A6A8FF.bnk
│   │   └── ...
│   └── streaming/
│       ├── 0000A8DE.wem
│       └── ...
├── english(us)/
│   ├── banks/
│   │   ├── 1216854A.bnk
│   │   │   └── 1216854A_media/    (embedded WEMs, if any)
│   │   └── ...
│   └── streaming/
│       └── ...
├── french(france)/
│   └── ...
└── ...
```

Extract from a standalone BNK:

```sh
pcktool dump Init.bnk -o init_media/
```

Filter by source ID:

```sh
pcktool dump Sounds.pck --id 0x1216854A -o single/
```

## Big-Endian PCK Support

Some PCK files (e.g. `Sounds_china.pck` from Halo Wars) use big-endian
byte order. The parser auto-detects this from the version field at offset 8
and handles the structural differences:

- **Compact header** — 5 fields (no external files LUT size)
- **Extended LUT entries** — 24 bytes per entry (extra 4-byte padding field)
- **UTF-16BE strings** in the language map

No flags or options needed — it just works.

## Building

```sh
cargo build --release
```

## License

MIT
