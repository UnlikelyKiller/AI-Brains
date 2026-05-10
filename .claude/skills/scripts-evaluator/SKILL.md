---
name: scripts-evaluator
description: >
  Robust multi-line script execution for Antigravity on Windows.
  Use this skill whenever you need to execute multi-line TypeScript or JavaScript
  using 'tsx' or 'node'. It prevents the "Node REPL" issue where the agent gets
  stuck in an interactive session.
---

# Scripts Evaluator

This skill provides a robust pattern for executing multi-line code blocks in 
PowerShell without triggering interactive REPLs or falling victim to quoting errors.

## The Pattern

When you need to execute more than one line of code, ALWAYS use the following 
pattern with the `scripts/eval-ts.ps1` utility:

```powershell
$env:TS_CODE = @'
// Your multi-line code here
// TypeScript or ESM is supported
'@; .\scripts\eval-ts.ps1
```

## Why this works
1. **PowerShell Here-String**: `@' ... '@` treats the entire block as a literal string,
   preserving newlines and quotes.
2. **Environment Variable**: Storing the code in `$env:TS_CODE` avoids the 8191-character 
   command line limit and parsing issues.
3. **eval-ts.ps1**: The script reads from the environment variable and pipes directly 
   into `npx tsx -`, ensuring Node reads from stdin and exits after execution.

## Usage Guidelines
- **Always** use this for Tandem scrapes, DB tests, or any complex logic.
- **Do not** attempt to use `tsx -e` for anything more complex than a single `console.log`.
- Ensure `scripts/eval-ts.ps1` exists in the workspace.
