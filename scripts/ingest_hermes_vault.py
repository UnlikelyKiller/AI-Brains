#!/usr/bin/env python3
"""
ingest_hermes_vault.py - Ingest all Hermes Obsidian vault notes into AI-Brains vault.

Usage: python3 ingest_hermes_vault.py

Reads all .md files from Hermes vault directory,
generates proper AI-Brains ingest JSON, and pipes to ai-brains ingest.
"""

import json
import os
import subprocess
import sys
import uuid
from pathlib import Path
from datetime import datetime, timezone

HERMES_DIR = Path("/mnt/c/Users/RyanB/Documents/Hermes")
AI_BRAINS_BIN = Path("/mnt/c/dev/AI-Brains/target/release/ai-brains")
VAULT_PATH = Path("/mnt/c/dev/ai-brains/vault.db")

# Project ID for the Hermes vault project (deterministic UUID v5 from vault path)
HERMES_PROJECT_ID = "66dd77f4-57cc-528b-972d-7478dc58ea8d"
# Single session for this ingestion batch
SESSION_ID = str(uuid.uuid4())
HARNESS_ID = "hermes-vault-ingest"

def get_markdown_files():
    """Find all .md files in the Hermes vault."""
    files = []
    for root, _, filenames in os.walk(HERMES_DIR):
        for fname in filenames:
            if fname.endswith(".md"):
                fpath = Path(root) / fname
                rel = fpath.relative_to(HERMES_DIR)
                files.append((rel, fpath))
    return sorted(files, key=lambda x: str(x[0]))

def read_file_safe(filepath):
    """Read a file with multiple encoding attempts."""
    for encoding in ["utf-8", "utf-8-sig", "cp1252", "latin-1"]:
        try:
            with open(filepath, "r", encoding=encoding) as f:
                return f.read()
        except UnicodeDecodeError:
            continue
    return ""

def create_ingest_json(rel_path, content):
    """Create an AI-Brains ingest request for a vault note."""
    # Use file path as deterministic memory_id
    memory_id = f"hermes-vault/{rel_path}"
    turn_id = str(uuid.uuid4())
    
    # Metadata header for context
    header = f"""---
source: hermes-vault/{rel_path}
type: knowledge_base_note
project: Hermes
---
"""
    full_content = header + content
    
    # Truncate extremely large files (AI-Brains has practical limits)
    if len(full_content) > 50000:
        full_content = full_content[:50000] + "\n\n[...truncated for size...]"
    
    request = {
        "session_id": SESSION_ID,
        "project_id": HERMES_PROJECT_ID,
        "harness_id": HARNESS_ID,
        "turn_id": turn_id,
        "role": "system",
        "content": full_content,
        "privacy": "CloudOk"
    }
    
    return request

def ingest_file(rel_path, filepath):
    """Ingest a single file into the vault."""
    content = read_file_safe(filepath)
    if not content.strip():
        return True, "empty"
    
    request = create_ingest_json(rel_path, content)
    json_bytes = json.dumps(request, ensure_ascii=False).encode("utf-8")
    
    try:
        result = subprocess.run(
            [str(AI_BRAINS_BIN), "--vault-path", str(VAULT_PATH), "ingest"],
            input=json_bytes,
            capture_output=True,
            timeout=30,
            cwd=str(AI_BRAINS_BIN.parent.parent)  # Run from AI-Brains dir
        )
        
        if result.returncode == 0:
            return True, "ok"
        else:
            stderr = result.stderr.decode("utf-8", errors="replace")[:200]
            return False, f"error: {stderr}"
    except subprocess.TimeoutExpired:
        return False, "timeout"
    except Exception as e:
        return False, f"exception: {e}"

def main():
    print(f"AI-Brains Hermes Vault Ingestion")
    print(f"Project ID: {HERMES_PROJECT_ID}")
    print(f"Session ID: {SESSION_ID}")
    print(f"Vault: {VAULT_PATH}")
    print(f"Source: {HERMES_DIR}")
    print("=" * 60)
    
    files = get_markdown_files()
    print(f"Found {len(files)} markdown files\n")
    
    success = 0
    failed = 0
    errors = []
    
    for i, (rel_path, filepath) in enumerate(files, 1):
        ok, msg = ingest_file(rel_path, filepath)
        status = "✅" if ok else "❌"
        print(f"  [{i:3d}/{len(files)}] {status} {rel_path} ({msg})")
        
        if ok:
            success += 1
        else:
            failed += 1
            errors.append((str(rel_path), msg))
    
    print(f"\n{'='*60}")
    print(f"Summary: {success} succeeded, {failed} failed")
    if errors:
        print(f"\nErrors:")
        for path, msg in errors[:10]:
            print(f"  {path}: {msg}")
    print(f"\nSession: {SESSION_ID}")
    print(f"Test recall with: ai-brains recall 'your query' --limit 5")

if __name__ == "__main__":
    main()
