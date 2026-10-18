# save2gb: CM 01/02 saves above 2 GB

*[Versão em português](README.pt.md)*

A fix for **Championship Manager 01/02 (3.9.68)** careers whose save grows past **2 GB**. Without it, the game can no longer load the career: an error box appears (`comp_stats.cpp 6000/6007`, `Friendly.cpp` and others) and loading hangs. With the fix, saves work up to **4 GB**.

- **It does not change the exe.** It is a small `ddraw.dll` you drop into the game folder.
- **The save format does not change.** A save of up to 2 GB written with the fix opens in any game.
- **Works with modified exes.** It was tested on the CLZ Leagues Patch (DB 25-26) V5.4, built with NickPatcher.
- **Safe on unknown exes.** Before acting, it checks the code it depends on. If your exe differs there, it changes nothing.

## Download

Get **`save2gb-1.0.zip`** from the [Releases](../../releases) page.

## Install

1. Copy `ddraw.dll` from the zip into the game folder, next to `cm0102.exe`.
2. Start the game. `save2gb.log`, in the same folder, says:
   - `ligado` (on): the fix is active;
   - `exe diferente...`: your exe changed the save code, and the DLL changed nothing.

**Remove:** delete `ddraw.dll`.

**Already using another `ddraw.dll`** (dgVoodoo, cnc-ddraw...)? Rename yours to `ddraw_chain.dll` and put this one in its place. The video goes through yours.

`cm0102_GDI.exe` does not use `ddraw.dll`, so the fix does not apply to it.

## How it was proven

| | Saving above 2 GB | Loading above 2 GB |
|---|---|---|
| Original 3.9.68 | index broken: every part beyond 2 GB gets position `0xFFFFFFFF` | **error, loading hangs** |
| With save2gb (original 3.9.68) | 2.55 GB save, index correct | **loads normally** |
| CLZ 25-26 (NickPatcher), without save2gb | — | **error** |
| CLZ 25-26 (NickPatcher), with save2gb | 2.45 GB save, index correct | **loads normally** |

## For patch maintainers

The cause, the container format, the three routines involved (`zopen` 0x9326e0, `zseek` 0x933080, `ztell` 0x933170) and three ways to integrate the fix into your patch are in [`docs/SAVE_2GB.en.md`](docs/SAVE_2GB.en.md). In short: ship the DLL, `LoadLibrary` it from your own DLL, or rewrite the three routines.

The source code is available on request: open an issue, or message **lucastmpacheco** on the [champman0102 forum](https://champman0102.net).

Tools in [`tools/`](tools):
- `assinatura_save.py` tells whether the DLL will act on an exe;
- `enchimento.py` creates the 2.2 GB filler used to test a save above 2 GB without playing for years.

## About

Part of the **CM 01/02 in Rust** project, which rebuilds the game routine by routine. Every rewritten routine is proven identical to the original before it ships. Thanks to the champman0102 community, to davdav, and to everyone keeping 01/02 alive.
