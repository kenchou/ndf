# ndf - Nice Disk Free

A modern, colorful disk usage utility written in Rust. `ndf` provides a clean and intuitive way to view disk space information with beautiful progress bars and multiple display modes.

## Features

- 🎨 **Colorful Progress Bars**: Visual representation of disk usage with red/green color coding
- 📊 **Multiple Display Modes**: Choose from normal, compact, or table formats
- 📏 **Adaptive Table Layout**: Automatically adjusts column widths for optimal display
- 🔍 **Mount Point Filtering**: Include or exclude specific mount points
- ⚡ **Fast and Lightweight**: Written in Rust for optimal performance
- 🎯 **Smart Filtering**: Automatically ignores overlay and snap mounts

## Installation

### From Source

```bash
git clone https://github.com/{{owner}}/{{repo}}.git
cd ndf
cargo build --release
cargo install --path .
```

## Usage

### Basic Usage

```bash
# Default table mode
ndf

# Specific display modes
ndf normal    # Detailed view with mount points
ndf compact   # One-line per disk
ndf table     # Formatted table (default)
```

### Filtering Options

```bash
# Show only specific mount points
ndf --only-mp "/"
ndf --only-mp "/,/home"

# Exclude specific mount points
ndf --exclude-mp "/snap,/tmp"
```

### Help

```bash
ndf --help
```

## Display Modes

### Table Mode (Default)

The table display mode is inspired by [duf](https://github.com/muesli/duf), providing a clean and organized view of disk information with adaptive column widths.

```text
┌───────┬─────────┬─────────┬────────────────────────────────────────────────────────┬──────────────┐
│ Mount │    Size │    Free │                         Usage                          │ Name         │
├───────┼─────────┼─────────┼────────────────────────────────────────────────────────┼──────────────┤
│ /     │ 926.35G │ 303.61G │ ▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓░░░░░░░░░░░░░░░░ 67% │ Macintosh HD │
└───────┴─────────┴─────────┴────────────────────────────────────────────────────────┴──────────────┘
```

### Normal Mode

```text
Macintosh HD @ /
▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓░░░░░░░░░░░░░░░░ 67%
```

### Compact Mode

```text
Macintosh HD: ▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓░░░░░░░░░░░░░░░░ 67%
```

## Color Coding

- 🟩 **Green**: Normal usage (< 80%)
- 🟥 **Red**: High usage (≥ 80%)

## Command Line Options

```text
Usage: ndf [OPTIONS] [mode]

Arguments:
  [mode]  Display mode: normal | compact | table [default: table]

Options:
      --only-mp <MOUNTPOINTS>     Show only specified mount points, comma separated
      --exclude-mp <MOUNTPOINTS>  Exclude specified mount points, comma separated
  -h, --help                      Print help
```

## Requirements

- Rust 1.70+ (for building from source)

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Acknowledgments

- Built with [clap](https://github.com/clap-rs/clap) for command-line parsing
- Uses [colored](https://github.com/mackwic/colored) for terminal colors
- System information provided by [sysinfo](https://github.com/GuillaumeGomez/sysinfo)
