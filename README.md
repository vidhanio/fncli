# `fncli`

An attribute macro to simplify writing simple command line applications.

## Example

```rust
#[fncli::cli]
fn main(a: i32, b: i32) {
    println!("{}", a + b);
}
```

```bash session
$ cargo run 1 2
3
```

```bash session
$ cargo run 1
missing argument `b: i32`
```

```bash session
$ cargo run 1 2 3
too many arguments (expected 2 arguments)
```

```bash session
$ cargo run 1 a
failed to parse argument `b: i32`: ParseIntError { kind: InvalidDigit }
```

## How It Works

Here is the expanded code:

```rust
fn main() {
    let (a, b) = {
        let mut args = std::env::args().skip(1);

        let tuple = (
            {
                let arg = args.next().expect("missing argument `a: i32`");
                i32::from_str(&arg).expect("failed to parse argument `a: i32`")
            },
            {
                let arg = args.next().expect("missing argument `b: i32`");
                i32::from_str(&arg).expect("failed to parse argument `b: i32`")
            },
        );

        if args.next().is_some() {
            panic!("too many arguments (expected 2 arguments)");
        }

        tuple
    };
    {
        println!("{}", a + b);
    }
}
```

The generated code is very simple, and not too different from how one would write the code by hand.
