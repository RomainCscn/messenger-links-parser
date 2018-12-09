# messenger-links-parser

A rust parser that output links sent in a Messenger conversation. Currently built as a REST api and a cli tool.

## Client

As well as this tool, I built a [web client](https://github.com/RomainCscn/messenger-parser-client) in Vue.js that allows you to search links and display some neat statistics.

## How to run the parser

- Install [Rust](https://www.rust-lang.org/en-US/install.html)
- Add a `message.json` file containing your Messenger conversation at the root of the project. Download this file from Facebook, specifying json format.
- Use `cargo build` to install dependencies and build the project.

### Server

- Use `cargo run server` to run the project as a web server.

Use any REST client (I personally use [Insomnia](https://insomnia.rest)) to try the api (using GET):
- `localhost:3000/search/all` to retrieve all links
- `localhost:3000/search/sender/Toto` to retrieve links sent by *Toto*
- `localhost:3000/search/site/youtube` to retrieve links containing *youtube*
- `localhost:3000/search/site/reddit/sender/Toto` to retrieve links containing *reddit* sent by *Toto*

You also can use query parameters to filter by date using `year`, `month` and `day` as parameters:

- `localhost:3000/search/all?year=2018&month=11&day=25` to retrieve links sent the 25/11/2018.

You can combine them as you want (e.g. only filter using year), the only constraint is that year is mandatory.

### Cli tool

- Use `cargo run cli message.json` to run the project as a cli tool using your message.json file.

To filter by sender, use the environment variable SENDER:

`SENDER=Foo cargo run cli message.json` to filter using Foo as the sender name.

To filter by site, use the site name as an argument after the path of message.json:

`cargo run cli message.json reddit.com` to filter only links containing reddit.com.

You can also combine both sender and site filter:

`SENDER=Foo cargo run cli message.json reddit.com` to filter using Foo as sender and links containing reddit.com.
