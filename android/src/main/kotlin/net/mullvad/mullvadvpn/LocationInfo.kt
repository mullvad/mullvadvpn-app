package net.mullvad.mullvadvpn

import android.content.Context
import android.view.View
import android.widget.TextView

import net.mullvad.mullvadvpn.model.Endpoint
import net.mullvad.mullvadvpn.model.GeoIpLocation
import net.mullvad.mullvadvpn.model.TransportProtocol
import net.mullvad.mullvadvpn.model.TunnelState

class LocationInfo(val parentView: View, val context: Context) {
    private val country: TextView = parentView.findViewById(R.id.country)
    private val city: TextView = parentView.findViewById(R.id.city)
    private val hostname: TextView = parentView.findViewById(R.id.hostname)
    private val protocol: TextView = parentView.findViewById(R.id.tunnel_protocol)
    private val inAddress: TextView = parentView.findViewById(R.id.in_address)

    private var endpoint: Endpoint? = null
    private var isTunnelInfoVisible = false

    var location: GeoIpLocation? = null
        set(value) {
            country.text = value?.country ?: ""
            city.text = value?.city ?: ""
            hostname.text = value?.hostname ?: ""
        }

    var state: TunnelState = TunnelState.Disconnected()
        set(value) {
            field = value

            when (value) {
                is TunnelState.Connecting -> {
                    endpoint = value.endpoint?.endpoint
                    isTunnelInfoVisible = true
                }
                is TunnelState.Connected -> {
                    endpoint = value.endpoint.endpoint
                    isTunnelInfoVisible = true
                }
                else -> {
                    endpoint = null
                    isTunnelInfoVisible = false
                }
            }

            updateTunnelInfo()
        }

    private fun updateTunnelInfo() {
        if (isTunnelInfoVisible) {
            showTunnelInfo()
        } else {
            hideTunnelInfo()
        }
    }

    private fun hideTunnelInfo() {
        protocol.text = ""
        inAddress.text = ""
    }

    private fun showTunnelInfo() {
        protocol.setText(R.string.wireguard)
        showInAddress(endpoint)
    }

    private fun showInAddress(endpoint: Endpoint?) {
        if (endpoint != null) {
            val transportProtocol = when (endpoint.protocol) {
                is TransportProtocol.Tcp -> context.getString(R.string.tcp)
                is TransportProtocol.Udp -> context.getString(R.string.udp)
            }

            inAddress.text = context.getString(
                R.string.in_address,
                endpoint.address.address.hostAddress,
                endpoint.address.port,
                transportProtocol
            )
        } else {
            inAddress.text = ""
        }
    }
}
