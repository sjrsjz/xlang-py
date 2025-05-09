#!/usr/bin/env python3
import os
import subprocess
import sys
from setuptools import setup, find_packages
from setuptools.command.install import install
from setuptools.command.develop import develop
from setuptools.command.build_py import build_py


class MaturinBuildCommand(build_py):
    """Build Rust extension module using maturin"""
    def run(self):
        try:
            # Call maturin to build
            print("Building Rust extension with maturin...")
            subprocess.check_call(['maturin', 'build', '--release'])
            # Optional: Uncomment the following line to build multi-platform wheels
            # subprocess.check_call(['maturin', 'build', '--release', '--compatibility', 'manylinux2014'])
        except subprocess.CalledProcessError as e:
            print(f"maturin build failed: {e}", file=sys.stderr)
            raise
        build_py.run(self)


class MaturinInstallCommand(install):
    """Ensure Rust code is built before installation"""
    def run(self):
        self.run_command('build_py')
        install.run(self)


class MaturinDevelopCommand(develop):
    """Ensure Rust code is built before development installation"""
    def run(self):
        self.run_command('build_py')
        develop.run(self)


# 从 pyproject.toml 读取元数据
try:
    import tomli
    with open("pyproject.toml", "rb") as f:
        pyproject = tomli.load(f)
    project_info = pyproject.get("project", {})
except (ImportError, FileNotFoundError):
    # 如果没有 tomli 或找不到文件，使用默认值
    project_info = {}

# 读取 README.md
try:
    with open('README.md', 'r', encoding='utf-8') as fh:
        long_description = fh.read()
except FileNotFoundError:
    long_description = project_info.get("description", "XLang Python bindings")

setup(
    name=project_info.get("name", "xlang-py"),
    version=project_info.get("version", "0.1.0"),
    author=project_info.get("authors", [{}])[0].get("name", "Your Name"),
    author_email=project_info.get("authors", [{}])[0].get("email", "your.email@example.com"),
    description=project_info.get("description", "XLang Python bindings"),
    long_description=long_description,
    long_description_content_type="text/markdown",
    url="https://github.com/yourusername/xlang-py",
    packages=find_packages(where="src_py"),
    package_dir={"": "src_py"},
    python_requires=project_info.get("requires-python", ">=3.7"),
    cmdclass={
        'build_py': MaturinBuildCommand,
        'install': MaturinInstallCommand,
        'develop': MaturinDevelopCommand,
    },
    classifiers=project_info.get("classifiers", [
        "Programming Language :: Python :: 3",
        "Programming Language :: Rust",
        "Development Status :: 3 - Alpha",
        "License :: OSI Approved :: MIT License",
    ]),
    # 将wheel文件包含在包中
    include_package_data=True,
)
