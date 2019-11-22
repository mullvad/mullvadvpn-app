package net.mullvad.mullvadvpn.model

import java.util.ArrayList

data class RelayTunnels(val wireguard: ArrayList<WireguardEndpointData>)
