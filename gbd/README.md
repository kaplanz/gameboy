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
* `break`,     `br`,   `b`: Set breakpoint.
* `capture`,   `ps`       : Capture a screenshot.
* `continue`,  `cont`, `c`: Resume execution.
* `delete`,    `del`      : Delete breakpoint.
* `disable`,   `dis`,  `d`: Disable breakpoint.
* `enable`,    `en`,   `e`: Enable breakpoint.
* `frequency`, `freq`, `d`: Change step frequency.
* `goto`,      `go`,   `g`: Goto address.
* `help`,              `h`: Print help.
* `ignore`,    `ig`       : Ignore breakpoint.
* `jump`,      `jp`,   `j`: Jump and execute.
* `list`,      `ls`,   `l`: List instruction.
* `load`,      `ld`       : Load register.
* `log`,       `lo`       : Change logging level.
* `quit`,              `q`: Quit emulator.
* `read`,      `rd`,   `r`: Read address.
* `reset`,     `res`      : Reset emulator.
* `step`,              `s`: Perform debugger step.
* `store`,     `sr`       : Store register.
* `write`,     `wr`,   `w`: Write address.

Use `help` for more information about how to use a command.
```

For further help with a specific command, pass that as an argument to help. For
example, `help break` will produce the following:

```
`break <ADDRESS>`

Set breakpoint at specified location.

Note that due to the SM83 CPU supporting multi-byte instructions, there
is a chance that the specified breakpoint will not occur upon an
instruction boundary. When this occurs, the breakpoint will NOT trigger.

Aliases: `br`, `b`
```

## License

For information regarding licensure, please see the project's [README][license].

<!--
  Reference-style links
-->

<!-- License -->
[license]: /README.md#license
