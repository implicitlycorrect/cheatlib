# cheatlib
Game hacking crate for windows inspired by [toy-arms](https://github.com/pseuxide/toy-arms)

## Features:
- internal
- external
- minhook | enables function hooking via the [minhook_sys](https://docs.rs/minhook-sys) crate

### Default Features:
```toml
default = ["internal"]
```

## TODO:
- [ ] Add proper documentation
- [ ] Linux support (maybe one day)

## Internal example:
### Cargo.toml
```toml
[lib]
crate-type = ["cdylib"]

[dependencies]
cheatlib = { git = "https://github.com/implicitlycorrect/cheatlib" }
```
### lib.rs
```rust
use cheatlib::*;

fn main() -> Result<()> {
    // cheatlib manages the console for debug builds
    println!("hello from DllMain!");
    Ok(())
}

dll_main!(main);
```
For a more detailed internal example there is [blaze](https://github.com/implicitlycorrect/blaze)

## External example:
### Cargo.toml
```toml
[dependencies]
cheatlib = { git = "https://github.com/implicitlycorrect/cheatlib", features = [
    "external",
], default-features = false }
```
### main.rs
```rust
use cheatlib::*;

fn main() -> Result<()> {
    let process = Process::from_name("cs2.exe")?;
    println!("found process: {}", process.id);

    let client = process.get_module_by_name("client.dll")?;
    println!("loaded client.dll {:#0x}", client.base_address);
    Ok(())
}
```