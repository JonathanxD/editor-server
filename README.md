[![Doc](https://docs.rs/editor-server/badge.svg)](https://docs.rs/editor-server)
[![Crate](https://img.shields.io/crates/v/editor-server.svg)](https://crates.io/crates/editor-server)
[![Github Release](https://img.shields.io/github/v/release/JonathanxD/editor-server?label=github%20release)](https://github.com/JonathanxD/editor-server/releases)
[![License: MIT](https://img.shields.io/crates/l/validbr)](https://opensource.org/licenses/MIT)

# editor-server

**editor-server** is a HTTP server which interfaces with a specified file. **editor-server** could be used for integration with applications which uses `$EDITOR` or `$VISUAL` for file editing.

It is important to understand that **editor-server** is not an editor in fact, it does simulate a text editor with four basic functionalities:

- Read all file contents
- Overwrite all file contents
- Save contents to file
- Reload contents from file

> Currently **editor-server** is being developed to be used by [Dracon IntelliJ Plugin](https://nest.pijul.com/Jonathan/Dracon) to integrate IntelliJ editor with `pijul record` command, but could be used in any kind of project to simulate a basic file editor.

## Usage

```bash
editor-server -p [port] [file]
```

### Starting editor-server

For example, to start an editor-server listening on port 7070 to read and write to a file named `example.json`:

```bash
editor-server -p 7070 example.json
```

### Reading file contents from buffer

**editor-server** caches file contents in memory, to read the cached contents, send a `GET` request to `/read` endpoint.

```http request
GET http://localhost:7070/read
```

It does respond with entire file contents stored in memory.

### Writing file contents to buffer

**editor-server** caches file contents in memory, to write contents to cache, send a `POST` request to `/write` endpoint with the contents in the body of request.

```http request
POST http://localhost:7070/read
{"name": "editor-server"}
```

It writes the content to buffer in memory and respond with the amount of bytes written.

### Saving file contents/flushing data

As pointed before, **editor-server** caches file contents in memory, to save the contents to file you must call `/save` endpoint.

```http request
GET http://localhost:7070/save
```

It does resets the contents of the file (all changes made directly to file are discarded) and writes the contents from the buffer, responding with the amount of bytes written.

### Reloading file contents

To reload file contents stored in the buffer (in other words, to load the file contents into the buffer), call `/reload` endpoint.

```http request
GET http://localhost:7070/reload
```

It does read file contents into the buffer and respond with the amount of bytes read.

### Closing the editor

To close the editor, call the `/close` endpoint, it does flushes all contents stored in buffer to the file, if this is not the desired behavior, just call `/reload` to discard buffer changes, before closing the editor.

```http request
GET http://localhost:7070/close
```

## Specifying PORT without -p parameter

**editor-server** supports port specification through `$EDITOR_SERVER_PORT` environment variable.

## Practical uses

**editor-server** will be used in [Dracon IntelliJ Plugin](https://nest.pijul.com/Jonathan/Dracon) as replacement for [copie](https://github.com/JonathanxD/copie). An **editor-server** will be launched with a random port and file contents will be read and written through REST API when file is changed in the [IntelliJ Editor](https://www.jetbrains.com/pt-br/idea/).