# `fncli`

An attribute macro to simplify writing simple command line applications.

## Example

```rust
#[fncli::cli]
fn main(a: i32, b: i32) {
    println!("{}", a + b);
}
```

```rust
$ cargo run 1 2
3
```

```rust
$ cargo run 1
missing argument: `b: i32`

USAGE:
    target/debug/examples/add <a: i32> <b: i32>
```

```rust
$ cargo run 1 2 3
too many arguments (expected 2)

USAGE:
    target/debug/examples/add <a: i32> <b: i32>
```

```rust
$ cargo run 1 a
failed to parse argument: `b: i32` (ParseIntError { kind: InvalidDigit })

USAGE:
    target/debug/examples/add <a: i32> <b: i32>
```

For a more complete example, see [the time elapsed example](examples/time_elapsed.rs).

## How It Works

```rust
fn main() {
    let (a, b) = {
        let mut args = std::env::args();

        let cmd = args.next().expect("should have a command name");

        let exit = |err: &str| -> ! {
            eprintln!("{err}");
            eprintln!();
            eprintln!("USAGE:\n    {cmd} <a: i32> <b: i32>");
            std::process::exit(1)
        };

        let tuple = (
            i32::from_str(
                &args
                    .next()
                    .unwrap_or_else(|| exit("missing argument: `a: i32`")),
            )
            .unwrap_or_else(|e| exit(&format!("failed to parse argument `a: i32` ({e:?})"))),
            i32::from_str(
                &args
                    .next()
                    .unwrap_or_else(|| exit("missing argument: `b: i32`")),
            )
            .unwrap_or_else(|e| exit(&format!("failed to parse argument `b: i32` ({e:?})"))),
        );

        if args.next().is_some() {
            exit("too many arguments (expected 2)");
        }

        tuple
    };

    {
        println!("{}", a + b);
    }
}
```

The generated code is very simple, and not too different from how one would write the code by hand.
