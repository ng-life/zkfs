# Zookeeper CLI Tool

A Rust-based command-line tool for interacting with Apache Zookeeper.

## Project Type
Rust CLI application

## Features
- Connect to Zookeeper using parameters or environment variables
- Support commands: ls, dir, cat, stat, rm, create/add, set
- Recursive delete with -r and force delete with -f options
- Multiple node types: persistent, ephemeral, sequential
- Read/write data from files or command-line arguments
- Command-line argument parsing with clap
- Asynchronous Zookeeper client operations
