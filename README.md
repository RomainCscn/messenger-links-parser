# messenger-links-parser
A rust parser that output links sent in a Messenger conversation. Currently built as a small REST api.

## How to run

- Install [Rust](https://www.rust-lang.org/en-US/install.html)
- Add a `message.json` file containing your Messenger conversation at the root of the project. Download this file from Facebook, specifying json format.
- Use `cargo run` to install dependencies, build and run the project.

Use any REST client (I personally use [Insomnia](https://insomnia.rest)) to try the api (using GET):
- `localhost:3000/search/all` to retrieve all links
- `localhost:3000/search/sender/Toto` to retrieve links sent by *Toto*
- `localhost:3000/search/site/youtube` to retrieve links containing *youtube*
- `localhost:3000/search/site/reddit/sender/Toto` to retrieve links containing *reddit* sent by *Toto*
