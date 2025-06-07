Organise this around subcommands:

```
dsync-cli
  host
    list (--discover)
    discover
  file
    add (--group [GROUP_NAME])
    remove (--group [GROUP_NAME]) // remove files from given group
    list (REMOTE) (--all) (--group [GROUP_NAME])
    sync [LOCAL-FILE-ID] [REMOTE-FILE-ID]
    unsync [LOCAL-FILE-ID] (REMOTE-FILE-ID) // if remote is not specified, then unsync from all remotes
  group
    create [GROUP_NAME]
    delete [GROUP_NAME]
    list (REMOTE) (--all)
```
