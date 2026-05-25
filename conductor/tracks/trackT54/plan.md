# Plan: Track T54 - Bridge Stderr Hardening

- [ ] **Phase 1: Child Process Refinement**
    - [ ] Identify all `Command::new("changeguard")` calls in `sync.rs`.
    - [ ] Update them to check `ctx.quiet`.

- [ ] **Phase 2: Stderr Suppression**
    - [ ] Implement conditional `.stderr(Stdio::null())` based on the quiet flag.
    - [ ] Ensure fatal/unrecognized errors are still reported or logged.

- [ ] **Phase 3: Verification**
    - [ ] Simulate ChangeGuard lock.
    - [ ] Verify `ai-brains sync query --quiet` is completely silent.
