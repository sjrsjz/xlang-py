#!/usr/bin/env python3
"""
Build script - Demonstrates how to use setup.py and maturin to build xlang-py project
"""
import os
import subprocess
import sys
import argparse


def main():
    parser = argparse.ArgumentParser(description="Build xlang-py project")
    parser.add_argument("--release", action="store_true", help="Build release version")
    parser.add_argument("--wheel", action="store_true", help="Build wheel package")
    parser.add_argument("--sdist", action="store_true", help="Build source distribution")
    parser.add_argument("--install", action="store_true", help="Install to current environment")
    parser.add_argument("--develop", action="store_true", help="Install in development mode")
    parser.add_argument("--all-platforms", action="store_true", help="Build wheels for all platforms")
    args = parser.parse_args()    # Add color output
    GREEN = "\033[92m" if sys.platform != "win32" else ""
    RED = "\033[91m" if sys.platform != "win32" else ""
    RESET = "\033[0m" if sys.platform != "win32" else ""

    try:
        if args.wheel or args.all_platforms:
            print(f"{GREEN}Building wheel package...{RESET}")
            cmd = ["maturin", "build"]
            if args.release:
                cmd.append("--release")
            if args.all_platforms:
                cmd.extend(["--compatibility", "manylinux2014"])
            subprocess.check_call(cmd)
        elif args.sdist:
            print(f"{GREEN}Building source distribution...{RESET}")
            subprocess.check_call(["python", "setup.py", "sdist"])
        elif args.install:
            print(f"{GREEN}Installing to current environment...{RESET}")
            subprocess.check_call(["python", "setup.py", "install"])
        elif args.develop:
            print(f"{GREEN}Installing in development mode...{RESET}")
            subprocess.check_call(["pip", "install", "-e", "."])
        else:
            # Default to using setup.py build
            print(f"{GREEN}Building with setup.py...{RESET}")
            cmd = ["python", "setup.py", "build_py"]
            subprocess.check_call(cmd)
            
        print(f"{GREEN}Build successful!{RESET}")
        
    except subprocess.CalledProcessError as e:
        print(f"{RED}Build failed: {e}{RESET}", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()
