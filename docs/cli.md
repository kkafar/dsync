Organise this around subcommands:

```
dsync-cli
  host
    list (--discover)
    discover
  file
    add (--group [GROUP_NAME])
    remove
    list (REMOTE) (--all)
    sync [LOCAL-FILE-ID] [REMOTE-FILE-ID]
    unsync [LOCAL-FILE-ID] (REMOTE-FILE-ID) // if remote is not specified, then unsync from all remotes
  group
    create [GROUP_NAME]
    delete [GROUP_NAME]
    list (REMOTE) (--all)
```
