Organise this around subcommands:

```
dsync-cli
  host
    list (--discover)
    discover
  file
    add (track)
    remove
    list (REMOTE) (--all)
    sync [LOCAL-FILE-ID] [REMOTE-FILE-ID]
    unsync [LOCAL-FILE-ID] (REMOTE-FILE-ID) // if remote is not specified, then unsync from all remotes
```
