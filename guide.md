# Power & Frequency Control

# NUMA

Check if a machine has multiple NUMAs.

```sh
lscpu | rg NUMA
# or
numactl -H
```

Good NUMA-aware applications have 0 numa_miss and numa_foreign.

```sh
❯ numastat  
                           node0
numa_hit              2876646598
numa_miss                      0
numa_foreign                   0
interleave_hit             11347
local_node            2876646598
other_node                     0
```

# Disable Speculative Execution Mitigations

Add kernel parameters to `GRUB_CMDLINE_LINUX_DEFAULT` in  `/etc/default/grub`, then call `grub2-mkconfig -o /boot/grub2/grub.cfg`.

`mitigations=off` is enough to disable most mitigations including spectre, mds, tsx for x86 and arm64.
If you want to manually list a few, use

```
nospectre_v1 nospectre_v2 pti=off mds=off tsx_async_abort=off
```

[Reference](https://github.com/torvalds/linux/blob/master/Documentation/admin-guide/kernel-parameters.txt#L2629)


# Disable Iptables
```
modprobe -rv ip_tables
```

For Docker,
```sh
# https://gist.github.com/talawahtech/ce2fe1f6a3e3851d15e912e0a4e93734#file-extreme-benchmark-environment-yaml-L212
# Configure and start docker with iptables support disabled
mkdir -p /etc/systemd/system/docker.service.d/
cat > /etc/systemd/system/docker.service.d/startup_options.conf <<- EOF
[Service]
ExecStart=
ExecStart=/usr/bin/dockerd -H fd:// --bridge=none --iptables=false --ip-forward=false --live-restore
EOF

systemctl daemon-reload
systemctl enable docker
systemctl start docker
```

# Perfect Locality

# Disable Syscall Auditing

```sh
auditctl -a never,task
# or
echo "-a never,task" > /etc/audit/rules.d/disable-syscall-auditing.rules
/sbin/augenrules --load
```

For docker, use `--security-opt seccomp=unconfined`.


# Interrupt Moderation - enp3s0
Configure interrupt coalescing.

Set rx-usecs and tx-usecs to 256us or higher.
If you are not confident about the number, use adaptive-rx. It comes with an overhead.

# Busy Polling and others

```
vm.swappiness=0
vm.dirty_ratio=80

net.core.somaxconn=2048
net.ipv4.tcp_max_syn_backlog=10000

net.core.busy_poll=1
net.core.default_qdisc=noqueue

# only when data fits in a single TCP packet
# net.ipv4.tcp_congestion_control=reno 
```

or edit `/etc/sysctl.conf` or add `/etc/sysctl.d/90-opt.conf`.

The busy polling should be used with SO_ATTACH_REUSEPORT_CBPF.

# Disable DHCP

AWS EC2 has a fixed private IP so dhclient is unnecessary.
Set the lifetime of the private IP to forever.

```sh
interface=eth0

dhclient -x -pf /var/run/dhclient-$interface.pid
ip addr change $( ip -4 addr show dev $interface | grep 'inet' | awk '{ print $2 " brd " $4 " scope global"}') dev $interface valid_lft forever preferred_lft forever
```

# Options That Might Not Work
