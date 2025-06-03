import os
import sys
import shutil
import platform

PLATFORM = platform.system()
if PLATFORM == "Darwin": PLATFORM = "macOS"
ARCH = platform.machine()


def pack(cmd: str):
    code = os.system(cmd)
    if code != 0: sys.exit(1)

    os.mkdir("tmp")
    
    shutil.copy(os.path.join("target", "release", "skye.exe" if PLATFORM == "Windows" else "skye"), "tmp")
    shutil.copy("LICENSE", "tmp")
    shutil.copytree("lib", os.path.join("tmp", "lib"))

    shutil.make_archive(
        os.path.join("publish", f"Skye-{ARCH}-{PLATFORM}"), 
        "zip", "tmp"
    )

    shutil.rmtree("tmp")


if os.path.exists("publish"):
    shutil.rmtree("publish")

os.mkdir("publish")

pack("cargo build --release")