# Local / remote server information

## peer_base_info

No matter whether its local deamon or remote one, the common information is as follows:

1. name: string,
2. hostname: string,
3. uuid: string (pk),

The address is kinda problematic, because it is mutable - the ip addresses even in local network can change.

~Therefore, I think that I should put the addresses in separate table.~

Yeah, its mutable, however putting it into another table creates burden to uphold invariant in business logic.
Namely I would need to assure that if the host is present the address is also present.
It might be easier to just update the big record?

This is something to do in next PRs.

## peer_addr_v4

1. uuid: string (pk)
2. ipv4_addr: string
3. discovery_time: uint64 (time since 01.01.1970)

## local_server_info

The same schema as for `peer_base_info`.

# Synced paths data

Basically what I need to store for each file is for the local paths:

1. **Absolute** file path (local!),
2. File **content** hash (SH1 should be enough),
3. (NOT SURE) The last modified date of the file for which SHA1 has has been computed against, (best in some absolute manner, e.g. seconds from EPOCH or something),
4. Maybe file_uuid (look at the next table for explanation) (it might be just an usize)

Then, synced files table must contain:

1. peer_uuid,
2. Some kind of identifier of a particular file (maybe path is enough, because it should be unique) or uuid?
  Maybe I should generate UUID for each file, that is independent of its content & only servers as a source of identitiy?
3. The identifier described above, but on the remote peer.

To be honest as a identifier just an unique number is enough. It should be autoincrement, to avoid repeating the number (ignore the possibility
of exhausting the usize).

Then I should be able to do two things:

1. Pull information on all files exposed by the remote peer,
2. Pull information on whether there is anything to update (observed paths changed on remote)

Another idea (for the future) is to add `push` mode, where if I change a file, the deamon notifies all the peers registered for this file (kinda webhook).
