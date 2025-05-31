# Local / remote server information

## peer_base_info

No matter whether its local deamon or remote one, the common information is as follows:

1. name: string,
2. hostname: string,
3. uuid: string (pk),

The address is kinda problematic, because it is mutable - the ip addresses even in local network can change.

Therefore, I think that I should put the addresses in separate table.

## peer_addr_v4

1. uuid: string (pk)
2. ipv4_addr: string
3. discovery_time: uint64 (time since 01.01.1970)

## local_server_info

The same schema as for `peer_base_info`.

# Synced paths data

TODO
