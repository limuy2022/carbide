#!/usr/bin/env python3
import os
import subprocess
import sys
from pathlib import Path

if len(sys.argv) < 2:
    print("Usage: release.py <version>")
    sys.exit(1)

version = sys.argv[1]

try:
    # Generate changelog
    subprocess.run(["scripts/changelog.py", "--tag", version], check=True)
    
    # Git operations
    subprocess.run(["git", "add", "-A", "."], check=True)
    subprocess.run(["git", "commit", "-m", f"chore(release): {version}"], check=True)
    subprocess.run(["git", "tag", "-a", f"v{version}", "-m", f"chore(release): {version}"], check=True)

except subprocess.CalledProcessError as e:
    print(f"Release process failed: {e}")
    os._exit(1)
