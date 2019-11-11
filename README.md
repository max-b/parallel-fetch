# Parallel Fetch

## Installation
Binaries for multiple platforms are available in `./bin`

### With cargo/rustup
Install nightly Rust according to [the directions](https://rustup.rs).

In this directory, run:
```
$ cargo build --release
```

## Usage
Fetching the image at http://i.imgur.com/z4d4kWk.jpg can be achieved as:
```
$ ./parallel-fetch --url http://i.imgur.com/z4d4kWk.jpg -o ./
```
Argument information is available with:
```
$ ./parallel-fetch --help
```

## Testing
Assuming a functional rust environment, tests can be run with:
```
$ cargo test
```

## Notes
- ETag header is assumed to be md5 hex string
- `async/.await` [just landed on stable](https://blog.rust-lang.org/2019/11/07/Async-await-stable.html), but some of the error handling still requires nightly
- Parallel fetch requests *could* be retried on specific errors
