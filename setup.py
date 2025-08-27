#!/usr/bin/env python3
"""
Setup script for PR Title Generator CLI tool.
"""

from setuptools import setup, find_packages
import os

# Read the README file
def read_readme():
    with open("README.md", "r", encoding="utf-8") as fh:
        return fh.read()

# Read requirements
def read_requirements():
    with open("requirements.txt", "r", encoding="utf-8") as fh:
        return [line.strip() for line in fh if line.strip() and not line.startswith("#")]

setup(
    name="pr-title-generator",
    version="1.0.0",
    author="Alessandro Paciello",
    author_email="alessandro.paciello@crutrade.com",
    description="A CLI tool to generate meaningful PR titles using ML models",
    long_description=read_readme(),
    long_description_content_type="text/markdown",
    url="https://github.com/alessandropac96/pr-title-generator",
    packages=find_packages(),
    classifiers=[
        "Development Status :: 4 - Beta",
        "Intended Audience :: Developers",
        "License :: OSI Approved :: MIT License",
        "Operating System :: OS Independent",
        "Programming Language :: Python :: 3",
        "Programming Language :: Python :: 3.7",
        "Programming Language :: Python :: 3.8",
        "Programming Language :: Python :: 3.9",
        "Programming Language :: Python :: 3.10",
        "Programming Language :: Python :: 3.11",
        "Topic :: Software Development :: Libraries :: Python Modules",
        "Topic :: Software Development :: Version Control :: Git",
    ],
    python_requires=">=3.7",
    install_requires=read_requirements(),
    entry_points={
        "console_scripts": [
            "generate-pr-title=main:main",
        ],
    },
    include_package_data=True,
    zip_safe=False,
)
