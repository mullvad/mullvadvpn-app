# Split tunneling

Split tunneling allows excluding selected apps from the VPN tunnel. These apps will communicate with the network as if Mullvad VPN was disconnected or not even running.

## Vocabulary

* **Split tunneling** - The name of the feature.
* **Excluded app** - An app that only communicates outside of the VPN tunnel.
* **Included app** - An app that only communicates inside the VPN tunnel (when the tunnel is up).
  This is the default for all apps until they have been explicitly excluded.
* **To exclude** - The act of enabling split tunneling for a specific app, excluding its traffic
  from the VPN tunnel.
* **To include** - The act of disabling split tunneling for a specific app, including its traffic
  in the VPN tunnel again.

## DNS

DNS is a bit problematic to exclude properly. Ideally DNS requests from excluded apps would
always go outside the tunnel, because that's what they would have done if Mullvad was disconnected
or not running. But this is very hard/impossible to achieve on some platforms.
One reason for this is that on some operating systems, programs call into a system service
for name resolution. This system service will then perform the actual DNS lookup.
Since all DNS requests then originate from the same process/system service, it becomes hard
to know which ones are for excluded apps and which ones are not. Because the DNS service is not
excluded, DNS lookups **will fail** in the connecting, disconnecting, and error
[states](architecture.md) whenever they must be sent through a tunnel.

Some definitions of terms used later to describe behavior:

* **In tunnel** - DNS requests are sent in the VPN tunnel. Firewall rules ensure they
    are not allowed outside the tunnel for non-excluded apps*.
* **Outside tunnel** - DNS requests are sent outside the VPN tunnel. Firewall rules ensure
    they cannot go inside the tunnel*.
* **LAN** - Same as **Outside tunnel** with the addition that the firewall rules ensure
    the destination can only be in private non-routable IP ranges*.

* **Default DNS** - Custom DNS is disabled. The app uses the VPN relay server (default gateway)
    as the DNS resolver.
* **Private custom DNS** - Custom DNS is enabled and the resolver IP is in a private IP range.
* **Public custom DNS** - Custom DNS is enabled and the resolver IP is not in a private IP range.
* **System DNS** - Means the DNS configured in the operating system (or given by DHCP).

*: On platforms where we have custom firewall integration. This is currently on desktop operating
  systems, and not mobile.

### Desktop platforms (Windows, Linux, and macOS)

| In-app DNS setting | Normal & Excluded app                          |
|-|------------------------------------------------|
| **Default DNS** | In tunnel (to relay)                           |
| **Private custom DNS** (e.g. 10.0.1.1) | LAN (to 10.0.1.1)<br/>**macOS**: Not supported |
| **Public custom DNS** (e.g. 8.8.8.8) | In tunnel (to 8.8.8.8)                         |

In other words: Normal and excluded processes behave the same. This is because DNS is typically
handled by a service, e.g. DNS cache on Windows or systemd-resolved's resolver on Linux, which is
not an excluded process.

For the sake of simplicity and consistency, requests to public custom DNS resolvers are also sent
inside the tunnel when using a plain old static `resolv.conf`, even though it is technically
possible to exclude public custom DNS in that case.

### Android

| In-app DNS setting | Normal app | Excluded app |
|-|-|-|
| **Default DNS** | In tunnel (to relay) | Outside tunnel (to system DNS) |
| **Private custom DNS** (e.g. 10.0.1.1) | LAN* (to 10.0.1.1) | Outside tunnel (to system DNS) |
| **Public custom DNS** (e.g. 8.8.8.8) | In tunnel (to 8.8.8.8) | Outside tunnel (to system DNS) |

*: The "Local network sharing" option must be enabled to actually allow access to these IPs.
Otherwise DNS won't work.

In other words: Excluded apps behave as if there was no VPN tunnel running at all.

## Other limitations

Several limitations exist that relate to interprocess communication. An app is excluded if its path
is excluded or if its its parent process is exluded. This can be problematic at times. For example,
opening a browser often typically tells the existing browser instance to open a new window, which
means the "excluded" status is not inherited.

On Linux, especially, where split tunneling isn't path-based at all, this means that the new browser
window will be forked off from a process that isn't excluded.

This model also implies other potentially unexpected behavior. For example, clicking a link in an
excluded app may (if there's no existing browser instance) open a browser window that _is_
unexpectedly excluded, simply because the parent is excluded.

The limitations due to IPC are perhaps especially noticeable on macOS, since WebKit relies on other
processes to render web pages. This means that many browsers, including Safari, cannot be excluded
from the VPN.
