# `fcli`

An attribute macro to simplify writing simple command line applications.

```rust
#[fcli::cli]
fn main(a: i32, b: i32) {
    println!("{}", a + b);
}
```

```bash
$ cargo run 1 2
3
```
