# rugby-cli

This frontend provides a command-line interface (CLI) to quickly launch an
emulator session right from your terminal. It has first-class support for all
main features, including a full interactive debugger.

## Build

Invoke `cargo build --release -pcli` from anywhere within the repository to
build this frontend. The following artifacts of interest will be produced:

```
./target/release/
├── rugby          # rugby-cli executable
├── ...
└── build/cli-*/out/
   ├── _rugby      # completions for zsh
   ├── _rugby.ps1  # completions for powershell
   ├── rugby.6     # manual documentation page
   ├── rugby.bash  # completions for bash
   ├── rugby.elv   # completions for elvish
   └── rugby.fish  # completions for fish
```

## Usage

### Command-line

To see usage information, run the `rugby` executable with the `-h` flag (use
`--help` for more detailed descriptions):

```
Emulate the Nintendo Game Boy

Usage: rugby [OPTIONS] [ROM]

Arguments:
  [ROM]  Cartridge ROM image file

Options:
      --conf <PATH>   Configuration file [env: RUGBY_CONF=] [default:
                      $XDG_CONFIG_HOME/rugby/config.toml]
  -l, --log <FILTER>  Logging level [env: RUGBY_LOG=]
  -h, --help          Print help (see more with '--help')
  -V, --version       Print version

Cartridge:
  -b, --boot <PATH>  Boot ROM image file
  -c, --check        Check cartridge integrity
  -f, --force        Force cartridge construction

Interface:
  -p, --palette <COLOR>  DMG color palette [possible values: autumn-chill,
                         blk-aqu, blue-dream, coldfire, coral, demichrome,
                         earth, ice-cream, legacy, mist, mono, morris,
                         purple-dawn, rustic, velvet-cherry]
  -s, --speed <FREQ>     Simulated clock speed [possible values: half, actual,
                         double, max]
  -x, --exit             Exit after loading cartridge
  -H, --headless         Run without UI
      --host <ADDR>      Link cable local address
      --peer <ADDR>      Link cable peer address

Debug:
      --doc <PATH>  Doctor logfile path
  -i, --gbd         Enable interactive Game Boy Debugger
      --win         Open debug windows
```

### Configuration

When run, the program will load persistent configuration options from the first
file found according to following precedence rules:
1. Command-line option `--conf=<PATH>`.
1. Environment variable `RUGBY_CONF`.
1. Default path `$XDG_CONFIG_HOME/rugby/config.toml`.

When options are specified in multiple locations, they will be applied with the
following precedence: cli > env > file. This means these options may be
overridden when running the program.

Any relative paths specified in this file are resolved relative to this file's
parent directory. Use `--conf` on the command-line to provide an alternate
location for this file.

The default configuration is as follows:

```toml
[cartridge]
# Check cartridge integrity
check = false
# Force cartridge construction
force = false

[interface]
# DMG color palette
#
# Value must be a named preset or customized as follows:
# palette = { custom = ["#222", "#666", "#aaa", "#ddd"] }
palette = "mono"
# Simulated clock speed
#
# Value must be a named preset or customized as follows:
# speed   = { fps = 90 }      # runs at 1.50x
# speed   = { hz  = 3145728 } # runs at 0.75x
speed   = "actual"

[hardware]
# Boot ROM image file
# boot = "path/to/dmg_boot.bin"
```

A customized example could be found [here][config].

### Debugging

If `rugby` is launched with `-i/--gbd`, the program will instead present the
Game Boy Debugger (GBD) prompt after initialization. In this mode, commands can
be run to control and monitor execution of the console. While running with GBD
enabled, CTRL-C could be used to interrupt emulation and present the prompt.

To list and get help with GBD, use the `help` command at the prompt:

```
Game Boy Debugger.

COMMANDS:
* `break`,     `br`,   `b`: Set breakpoint.
* `continue`,  `cont`, `c`: Resume execution.
* `delete`,    `del`      : Delete breakpoint.
* `disable`,   `dis`,  `d`: Disable breakpoint.
* `enable`,    `en`,   `e`: Enable breakpoint.
* `frequency`, `freq`, `d`: Change step frequency.
* `goto`,      `go`,   `g`: Goto address.
* `help`,              `h`: Print help.
* `ignore`,    `ig`,      : Ignore breakpoint.
* `jump`,      `jp`,   `j`: Jump and execute.
* `list`,      `ls`,   `l`: List instruction.
* `load`,      `ld`,      : Load register.
* `log`,       `lo`,      : Change logging level.
* `print`,     `ps`,      : Print a screenshot.
* `quit`,              `q`: Quit emulator.
* `read`,      `rd`,   `r`: Read address.
* `reset`,     `res`,     : Reset emulator.
* `step`,              `s`: Perform debugger step.
* `store`,     `sr`,      : Store register.
* `write`,     `wr`,   `w`: Write address.

Use `help` for more information about how to use a command.
```

## License

For information regarding licensure, please see the project's [README][license].

<!-- Reference-style files -->
[config]:  ./docs/config.toml
[license]: /README.md#license
