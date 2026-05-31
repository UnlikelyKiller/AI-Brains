# Track T60 Plan: Install MinGW-w64 Cross-Compilation Toolchain

## Phase 1: Install Packages
```bash
sudo apt-get update
sudo apt-get install -y gcc-mingw-w64-x86-64 binutils-mingw-w64-x86-64
```

## Phase 2: Configure Cargo Linker
Create `~/.cargo/config.toml`:
```toml
[target.x86_64-pc-windows-gnu]
linker = "x86_64-w64-mingw32-gcc"
ar = "x86_64-w64-mingw32-ar"
```

## Phase 3: Verify Toolchain
```bash
which x86_64-w64-mingw32-gcc
x86_64-w64-mingw32-gcc --version
```

## Phase 4: Build Test
```bash
cargo build --target x86_64-pc-windows-gnu -p ai-brains-cli
```

## Phase 5: Create Skill Documentation
Create `.agents/skills/ai-brains/skill.md` with setup and operational instructions.

## Phase 6: Register in Conductor
Add T60 entry to `conductor/conductor.md` track registry.

## Phase 7: Branch + Commit + Push
Branch: `track-t60-mingw-w64-toolchain`
