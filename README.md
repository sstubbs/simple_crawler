# Simple Crawler
A very simple crawler library
## Quickstart
1. Build container
    ```shell
    docker build -t simple-crawler .
    ```
2. Run tests
    ```shell
    docker run -v `pwd`:/usr/src/app simple-crawler cargo test --release
    ```
3. Read the docs
   ```shell
   cargo doc --open
   ```
## Known issues
- This runs in memory for performance reasons. Seeing as it's built in rust memory usage is minimal but if using it
  on a site with 1000 or more urls stack overflow can occur if not run in release mode. Another option is to increase
  stack limit.