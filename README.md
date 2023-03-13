# battlesnake

Rust battle snake using simple graph searches.

## Setup and testing
### Start server:
> cargo run

### Start unit tests:
> cargo test

### Integration tests:
https://jfgodoy.github.io/battlesnake-tester/ provides a nice testing library.
Because battlesnake is running on localhost and the testing page is running on a diferent origin, you will need to relax your browser's content security policy.
In windows you can do this by running:
> chrome.exe --disable-web-security --user-data-dir="C:\Windows\Temp"
