#!/usr/bin/env python3
import subprocess
import sys

try:
    cpu = sys.argv[1]
except IndexError:
    print("Usage: docker-build.py <cpu>")
    sys.exit(1)

tag = f"limuy/carbide:{cpu}"
subprocess.run([
    "docker", "buildx", "build",
    "--load",
    "--tag", tag
], check=True)

print(f"Image tag: {tag}")
