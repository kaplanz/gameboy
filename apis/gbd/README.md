# rugby-gbd

The Game Boy Debugger, or GBD (a play on GDB, the GNU Debugger), is a debugging
framework for the `rugby` emulator. It provides an interactive command-line
debugging interface similar to its inspiration. When active, emulation will be
paused, and commands can be provided at the prompt.

## Usage

To list and get help with GBD, use the `help` command at the prompt:

```
Game Boy Debugger.

COMMANDS:
* `break`,     `br`,   `b`: Set a breakpoint.
* `capture`,   `ps`       : Capture a screenshot.
* `continue`,  `cont`, `c`: Continue execution.
* `delete`,    `del`      : Delete a breakpoint.
* `disable`,   `dis`,  `d`: Disable a breakpoint.
* `enable`,    `en`,   `e`: Enable a breakpoint.
* `frequency`, `freq`, `f`: Change the step unit.
* `goto`,      `go`,   `g`: Goto an address.
* `help`,              `h`: Print help.
* `ignore`,    `ig`       : Ignore a breakpoint.
* `info`,              `i`: Print debugger info.
* `jump`,      `jp`,   `j`: Jump and continue.
* `list`,      `ls`,   `l`: List the current instruction.
* `load`,      `ld`       : Load from a register.
* `log`,       `lo`       : Change the logging level.
* `quit`,              `q`: Quit the program.
* `read`,      `rd`,   `r`: Read from an address.
* `reset`,     `res`      : Reset the console.
* `serial`,    `sx`       : Perform serial I/O.
* `step`,              `s`: Execute a single step.
* `store`,     `sr`       : Store to a register.
* `write`,     `wr`,   `w`: Write to an address.

Use `help` for more information about how to use a command.
```

For further help with a specific command, pass that as an argument to help. For
example, `help break` will produce the following:

```
`break <ADDRESS>`

Set a breakpoint at the specified location.

Note that due to the SM83 CPU supporting multi-byte instructions, there
is a chance that the specified breakpoint will not occur upon an
instruction boundary. When this occurs, the breakpoint will NOT trigger.

Aliases: `br`, `b`
```

## Progress

- [x] Breakpoints
  - [x] Instruction address (PC)
  - [ ] Instruction opcode
  - [ ] I/O operation (read/write)
  - [ ] Softbreak (`ld b, b`)
  - [ ] Dynamic condition
- [x] Memory peek/poke
- [x] Register manipulation
  - [ ] APU
  - [x] CPU
  - [x] Interrupts
  - [x] PPU
  - [x] Serial
  - [x] Timer
- [x] Peripheral control
  - [x] Serial interface

## License

For information regarding licensure, please see the project's [README][license].

<!--
  Reference-style links
-->

<!-- License -->
[license]: /README.md#license
