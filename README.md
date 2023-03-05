# Metal Tuna

A network tuning diagnosis tool.

Checks are largely based on [the article in talawah.io](https://talawah.io/blog/extreme-http-performance-tuning-one-point-two-million/#_4-disabling-iptables-netfilter).


```
Disable Speculative Execution Mitigations
✗ Spectre v1 + SWAPGS
✗ Spectre v2
✗ Spectre v3/Meltdown
✗ MDS/Zombieload
✗ TSX Asynchronous Abort

Disable Iptables
✗ ip_tables
✗ ip6_tables
✗ arp_tables
✗ ebtables

Perfect Locality
? SO_ATTACH_REUSEPORT_CBPF
? RSS: Receive Side Scailing - disable irqbalance
? RSS: Receive Side Scailing - set affinity

..
✗ XPS: Transmit Packet Steering
```
