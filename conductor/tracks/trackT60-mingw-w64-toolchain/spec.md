# Track T60: Install MinGW-w64 Cross-Compilation Toolchain

## Context

The AI-Brains project targets `x86_64-pc-windows-gnu` for Windows builds (native Windows executable without MSVC dependency). The Rust target is installed (`rustup target list --installed` shows `x86_64-pc-windows-gnu`), but the actual MinGW-w64 C toolchain is missing from WSL.

Current state:
- `rustup target list --installed` → `x86_64-pc-windows-gnu` ✅
- `which x86_64-w64-mingw32-gcc` → not found ❌
- `~/.cargo/config.toml` → does not exist ❌
- `/usr/bin/x86_64-w64-mingw32*` → none exist ❌

Without the linker, `cargo build --target x86_64-pc-windows-gnu` fails with:
```
error: linker `x86_64-w64-mingw32-gcc` not found
```

## Requirements

1. **R1**: Install `gcc-mingw-w64-x86-64` and `binutils-mingw-w64-x86-64` on WSL
2. **R2**: Create `~/.cargo/config.toml` with linker configuration for `x86_64-pc-windows-gnu`
3. **R3**: Verify `cargo build --target x86_64-pc-windows-gnu` succeeds for `ai-brains-cli`
4. **R4**: Document the setup in `.agents/skills/ai-brains/skill.md` for future agents

## Technical Design

### Package Selection
Ubuntu packages needed:
- `gcc-mingw-w64-x86-64` — GNU C compiler for Win64
- `binutils-mingw-w64-x86-64` — Cross-binutils (linker, ar, etc.)

Optionally also install:
- `g++-mingw-w64-x86-64` — if any C++ dependencies are added later

### Cargo Configuration
Create `~/.cargo/config.toml`:
```toml
[target.x86_64-pc-windows-gnu]
linker = "x86_64-w64-mingw32-gcc"
ar = "x86_64-w64-mingw32-ar"
```

### Verification Steps
1. `which x86_64-w64-mingw32-gcc` — confirms linker in PATH
2. `x86_64-w64-mingw32-gcc --version` — confirms working
3. `cargo build --target x86_64-pc-windows-gnu -p ai-brains-cli` — full build succeeds
4. `file target/x86_64-pc-windows-gnu/release/ai-brains.exe` — confirms PE32+ executable

## Files to Modify / Create

- **New**: `~/.cargo/config.toml` — Cargo linker configuration
- **New**: `.agents/skills/ai-brains/skill.md` — Agent skill documentation
- **Update**: `conductor/conductor.md` — Add T60 to track registry

## Verification Plan

1. Run `sudo apt-get install -y gcc-mingw-w64-x86-64 binutils-mingw-w64-x86-64`
2. Run `which x86_64-w64-mingw32-gcc`
3. Run `cargo build --target x86_64-pc-windows-gnu -p ai-brains-cli`
4. Verify `target/x86_64-pc-windows-gnu/release/ai-brains.exe` exists
5. Run `cargo test --target x86_64-pc-windows-gnu -p ai-brains-cli` — tests pass

## Notes

- This is a **WSL-side system dependency**, not a code change.
- The existing `Cargo.toml` already has `windows` crate dependency; MinGW provides the C runtime and linker it needs.
- Install is idempotent — `apt-get install` can be re-run safely.
