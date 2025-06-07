# What is this project?

> [!important]
> Just found out that there is such thing as `rsync`... Therefore,
> this project aims to implement some subset of `rsync` functionality.
> It looks however that it'll have some unique features (other deamons discovery etc.).

Sync filesystem contents between given tree mount points.

E.g. given following file hierarchy on host A:

```
/some-path/.../sync-root-A/
    subdir-1/
        content-1
    content-2
```

when this file tree is synced with mount point `/some-path-2/../sync-root-B` I want
to just copy over all the files there.

Basically this looks like a Git a bit (or just a VCS) ;D Yeah, but there are going to be
at least few key differences:

1. I don't want to version the files & I do not need to "restore" older versions.
2. I don't need such conflicts resolution algorithms.

So what when there is a conflict? Manual resolution for startes. Might think of something
more clever later.

Client on given host should be able to read files available on another machine and
request conflicting files manually.

So basically I want to be able to read file list manifested by another client & select files
I want to download.

If there is a conflict user would be required to specify manually which file he wants.
Potentially we might introduce functionality of creating backup files.

On given host I want to be able to specify whole directories (and whole subtrees) to expose.

I want also to be able to expose singular paths.

# How is it going to work?

Each host must be able to discover other hosts and synchornize.

To achieve this there could be a deamon running in the background on predefined port.
Alternatively I could try to implement [Service location protocol](https://en.wikipedia.org/wiki/Service_Location_Protocol).

How do I discover hosts on local network, btw? `nmap -sP <addr>/<mask-n-bits>`, e.g. `nmap -sP 192.168.100.1/24` & parse the output.

Okay, let's now assume, that I've managed to discover clients with the `dsync`. What now? I need to store in local storage
the exposed file paths / file hierarchy roots.

Then, I need to be able to request exposed file paths from other hosts & be able to request their mount on local disk.

And that is basically it.

Additionally I want to be able to sync with ad-hoc clients, e.g. when a external disk is connected to host I want
to be able to treat some indicated file hierarchy as the sync-root and sync with it!

## How grouping should work

Each file / directory should be either "standalone" or "belonging to a group".
Then file can be added to a group (or multiple groups) and then synchronized by groupid / name.
There should be no problem with mapping differently named groups for each other across different hosts.

There should also be possibility of syncing a group across local host (duplicate files in many places, backup e.g.).

# System design

## Host infrastructure

Each host will have running daemon / server on some port `PN`.

To interface with the server there will be CLI client & in the future Web client.

## Server identification

Each server will have unique ID that should be unchanged - if changed it'll be treated as new identity by peers.
Each server just generates its own uuid.

## Discovery

Each server listens for connections on predetermined & agreed upon port number `PN`.
When requested, server tries to discover other servers in LAN by using `nmap` (for host discovery) & then sending `Hello` message & waiting up to 10s (timeout should be configurable).
If it receives a response - a peer is discovered & should be cached locally for use in later requests.

## File transfer

For the sake of fun I'll came up with custom protocol, however the program should be written in such way, that I can
replace it with some already functioning protocol someday.

1. I want to sync only the files that have changed.
2. I want to have partial transfer -> that is partition each file into chunks & transfer the chunks.
