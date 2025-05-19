# What is this project?

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

Okay, let's now assume, that I've managed to discover clients with the `dsync`.

