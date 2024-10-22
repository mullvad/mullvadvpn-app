pub fn check_for_leaks() {
    // TODO: When do we run this?
    // After connecting?
    // Periodically?
    // Whenever something changes? (interface, connection state, dns server, etc)
    // All of the above?

    // TODO: Figure out which interface(s) to bind to

    // TODO: get connection check config
    // http get https://am.i.mullvad.net/config

    // TODO: For each interface:

    // TODO: send an ICMP ping (to the relay?)
    // TODO: how to see if the pings are actually going outside the tunnel?

    // TODO: send a DNS request to leak check endpoint
    // TODO: will the service be able to handle all of the mullvad users constantly doing leak
    // checks

    // TODO: query DNS leak checker HTTPS endpoint

    // TODO: query https://ipv4.am.i.mullvad.net/
    // TODO: query https://ipv6.am.i.mullvad.net/
}
