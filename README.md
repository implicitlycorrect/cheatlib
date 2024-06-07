# cheatlib
Game hacking crate for windows inspired by [toy-arms](https://github.com/pseuxide/toy-arms)

TODO:
- [ ] Finish Process struct impl (file: src/process.rs)

## Features:
- internal
- external
- patternscan | enables scanning for patterns in a Module using the [patternscan](https://docs.rs/patternscan) crate
- minhook | enables function hooking via the [minhook_sys](https://docs.rs/minhook-sys) crate
- console | add on for internal feature that makes dll_main! macro allocate and deallocate a console before calling the provided function

### Default Features:
```toml
default = ["internal", "patternscan"]
```

## Simple usage (internal):
```rust
use anyhow::Result;

fn main() -> Result<()> {
    println!("hello from DllMain!");
    Ok(())
}

cheatlib::dll_main!(main);
```
