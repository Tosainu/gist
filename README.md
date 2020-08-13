gist
===

GitHub Gist command-line tool.

## Usage

See `gist --help` or `gist <command> --help` for more details.

### Login

`gist` command supports [OAuth2 device flow](https://docs.github.com/en/developers/apps/authorizing-oauth-apps#device-flow).

    $ gist login <client id of your oauth apps>
    open https://github.com/login/device and enter 'ABCD-1234'
    Success!

OAuth2 access token will store in `~/.config/gist/config.json`.

    $ cat ~/.config/gist/config.json
    {
      "Tosainu": {
        "type": "oauth",
        "value": "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"
      }
    }

### Upload the files

    $ gist upload <FILES>...

With `-s` option, files will upload as secret Gist.

    $ gist upload -s <FILES>...

### List uploaded Gists

    $ gist list
    https://gist.github.com/0fd4272fa909d46356d8acf35955f4e8
    https://gist.github.com/366c61c5353dbdded2ada3207cb2dfc3
    https://gist.github.com/ae676c1cc6f159cb0c7677099a6233bc Dockerfile for https://github.com/metashell/metashell
    https://gist.github.com/163c24cb69ccf4dae6f91379b9ddfe75
    https://gist.github.com/367e35aed49a2590d4f78fbca9e805c9 Brainf**k compiler (bf -> LLVM IR) and interpreter.
    (...)

To list @octocat's Gists, you can use `-u <username>` option.

    $ gist list -u octocat
    https://gist.github.com/6cad326836d38bd3a7ae Hello world!
    https://gist.github.com/0831f3fbd83ac4d46451
    https://gist.github.com/2a6851cde24cdaf4b85b
    https://gist.github.com/9257657 Some common .gitignore configurations
    https://gist.github.com/1305321
    https://gist.github.com/1169854
    https://gist.github.com/1169852
    https://gist.github.com/1162032

## Installation

    $ git clone https://github.com/Tosainu/gist.git
    $ cd gist
    $ cargo install --path .

## Related Projects

- [gist](https://github.com/defunkt/gist)

## License

[MIT](https://github.com/Tosainu/gist/blob/master/LICENSE)
