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
Assuming a functional nightly rust environment, tests can be run with:
```
$ cargo test
```

## Notes
- ETag header is assumed to be md5 hex string
- I've opted for a fairly basic error handling approach, where everything just returns a `Result<T, Box<dyn Error>>`
  - `Box<dyn Error>` is a trait object (rust's version of runtime polymorphism/dynamic dispatch)
  - That allows me to return any error without needing to worry about type conversions
  - *However*, it does so at the sacrifice of type expressivity (the compiler can't reason about what the underlying type is), and it still requires nightly
  - In a more mature project, I would switch to a custom Error type where I would manually define conversions
- `async/.await` [just landed on stable](https://blog.rust-lang.org/2019/11/07/Async-await-stable.html), but as mentioned above, this particular error handling approach still requires nightly
- An improvement could be retrying parallel fetch requests on specific (likely network/server) errors
- This implementation requires you to specify up front how many parallel fetches to attempt, and then divides the file up into that many chunks and executes those fetches all at once
  - I *believe* given the problem statement, and that currently no retrying is done, this is the most straightforward and performant approach
  - *However*, in a more mature project, if intermittent network failures were a concern and retrying was implemented, very large files might be more effectively downloaded by a worker pool each grabbing a fixed (likely smaller) size chunk
  - That way, if a particular fetch failed, it wouldn't necessarily have to redo the work of downloading from the beginning of a very large chunk
