# Parallel Fetch

## Installation
Binaries for multiple platforms are available in `./bin`

### With cargo/rustup
Install Rust according to [the directions](https://rustup.rs).

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
- `async/.await` [just landed on stable](https://blog.rust-lang.org/2019/11/07/Async-await-stable.html), but things are still getting sorted out a little bit, so a few of the crates I'm using are alpha (`reqwest` and `tokio`)
- An improvement could be retrying parallel fetch requests on specific (likely network/server) errors
- This implementation requires you to specify up front how many parallel fetches to attempt, and then divides the file up into that many chunks and executes those fetches all at once
  - I *believe* given the problem statement, and that currently no retrying is done, this is the most straightforward and performant approach
  - *However*, in a more mature project, if intermittent network failures were a concern and retrying was implemented, very large files might be more effectively downloaded by a worker pool each grabbing a fixed (likely smaller) size chunk
  - That way, if a particular fetch failed, it wouldn't necessarily have to redo the work of downloading from the beginning of a very large chunk
