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

TODO
