#!/usr/bin/env python3
import sys
import subprocess
from pathlib import Path

try:
    with open("changelog.md", "w") as f:
        subprocess.run(
            ["git", "cliff", "a69e8b609625b67a3e52e18f73ba5d0f49ceb7c3..HEAD"] + sys.argv[1:],
            check=True,
            stdout=f
        )
except subprocess.CalledProcessError as e:
    print(f"Error generating changelog: {e}")
    sys.exit(1)
