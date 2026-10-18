# Saves above 2 GB (up to 4 GB)

Fix for CM 01/02 3.9.68 careers whose save goes beyond 2 GB. Made in batch 158 (2026-10-17) and distributed in two ways:
- inside the `ddraw.dll` of the CM 01/02 in Rust project (private repository);
- in a **save-only DLL** (this repository, release `save2gb-1.0.zip`), made for modified exes (NickPatcher, CLZ and others).

## The problem

With the save above 2 GB, the game no longer loads the career. An error box appears with the source file of the part being read, such as `comp_stats.cpp 6000/6007` or `Friendly.cpp`, and loading hangs.

The workaround going around the forum was moving `.tmp` files by hand. It is no longer needed.

## The cause

The `.sav` (and the quick start `.qst`) is SI's own container, `si/code/zipdir.cpp`:

| Part | Content |
|---|---|
| bytes 0–11 | type (3 = uncompressed, 4 = RLE; the old 1/2 become 3/4), version, number of items |
| from byte 12 | index: up to 1000 items of 0x10c bytes = position (u32), size (u32), name (260 bytes) |
| after that | each item's data, one after the other |

The index stores positions as **unsigned 32-bit**, which go up to 4 GB. What cannot cope is the code: three routines use the CRT's `fseek`/`ftell`, which are **signed 32-bit** and only go up to 2 GB.

| Routine | Address | What goes wrong above 2 GB |
|---|---|---|
| `zopen` | 0x9326e0 | when writing, `fseek(end)` + `ftell` returns −1: the item gets position 0xFFFFFFFF in the index and the save is already broken when written; when reading, the signed `fseek(position)` fails |
| `zseek` | 0x933080 | compares positions as signed and seeks with a 32-bit `fseek` |
| `ztell` | 0x933170 | 32-bit `ftell` |

The other container functions (`zread` 0x932c30, `zwrite` 0x932e10, `zclose` 0x932b00, pack 0x933610, unpack 0x933320) read and write sequentially, or only go back to the start of the file, and need no fix. The CRT functions `_findfirst` and `GetDiskFreeSpace` do not limit the save either.

## The fix

The three routines were rewritten in Rust (`src/conteiner.rs`):
- positions treated as unsigned;
- seek and tell through Windows' `SetFilePointer`, with the 64-bit high part, repeating what the CRT's `fseek`/`ftell` do with the `FILE`: flushing the buffer, read and write flags, end of file in the file table, discounting the buffer in the position;
- **the format does not change**: a save of up to 2 GB written with the fix opens in a game without it;
- **above 4 GB**, the item is refused as if the container were full, with a line in the log, without damaging the file. An extended format comes later.

## The save-only DLL

The release ships a `ddraw.dll` of ~190 KB.

**Install:**
1. Copy the `ddraw.dll` into the game exe's folder.
2. Done: the exe is not altered.

**Remove:** delete the `ddraw.dll`.

**What it does when loaded:**
1. Computes the signature (64-bit FNV-1a) of the code it depends on: the whole container (0x9326e0–0x933860) and the start of 14 CRT and game functions the routines call.
2. If the signature is 3.9.68's (`232589cfe29f6240`), it replaces the entry of the three routines with a jump to the Rust versions and writes "ligado" (on) to `save2gb.log`.
3. If not, it **changes nothing** and writes the signature it found to `save2gb.log`.
4. `DirectDrawCreate`, the only function the exe imports from `ddraw.dll`, is forwarded to Windows' `ddraw.dll`.

**Another video wrapper** (dgVoodoo, cnc-ddraw etc.): rename its `ddraw.dll` to `ddraw_chain.dll`, in the same folder. Ours forwards to it.

**Check:** `python tools/assinatura_save.py <exe>` (needs `pip install pefile`) shows an exe's signature. Results:

| Exe | Signature | Does the DLL act? |
|---|---|---|
| `cm0102.exe` 3.9.68 and the project's portable and 1080 copies | 232589cfe29f6240 | yes |
| CLZ Leagues Patch (DB 25-26) V5.4, with NickPatcher: `cm0102.exe` and "Tema Black" | 232589cfe29f6240 | yes |
| `cm0102_GDI.exe` 3.9.68 | c7f8f4659479ddda | no: the GDI exe's addresses are different, and it does not even load `ddraw.dll` |

## For patch maintainers

There are three ways to integrate:
1. **Ship the DLL as is**, alongside the patch.
2. **Load the DLL from your own code.** The hooking happens on load, whatever the file name. Rename it to, for example, `save2gb.dll` and call `LoadLibrary("save2gb.dll")` from your DLL, such as `NickDLL`. In that case its `DirectDrawCreate` is not used.
3. **Rewrite the three routines in your code**, with the description above and `src/conteiner.rs` as reference: in `fseek`/`ftell`, swap the `long` for an unsigned position and use 64 bits.

If your patch changes bytes in 0x9326e0–0x933860, the signature does not match and the DLL stays inert. In that case, get in touch.

## How to test

The save takes along every `*.tmp` in the game folder, in name order. A large file whose name comes first pushes the rest beyond 2 GB:
1. With the career loaded, create the filler: `python tools/enchimento.py <game folder> 2200`, which creates a 2.2 GB `0000_enchimento.tmp`. It must be created after loading, because the game deletes the `.tmp` files when loading and when creating a career.
2. Save under another name and load that save: with the fix, it opens.
3. Without the fix, loading the same save fails.
4. Delete the filler: `python tools/enchimento.py <game folder> --apagar`.


Results from 2026-10-17:
- **2.55 GB save**, with 187 items beyond 2 GB: loaded with the fix; without it, error `Friendly.cpp`;
- **2.38 GB `.qst`:** the original writes 0xFFFFFFFF in the positions beyond 2 GB, and with the fix the index comes out right.
