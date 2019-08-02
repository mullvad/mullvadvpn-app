package net.mullvad.mullvadvpn

import android.view.View
import android.widget.TextView

import net.mullvad.mullvadvpn.model.GeoIpLocation
import net.mullvad.mullvadvpn.model.TunnelState

class LocationInfo(val parentView: View) {
    private val country: TextView = parentView.findViewById(R.id.country)
    private val city: TextView = parentView.findViewById(R.id.city)
    private val hostname: TextView = parentView.findViewById(R.id.hostname)
    private val protocol: TextView = parentView.findViewById(R.id.tunnel_protocol)

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
                is TunnelState.Connecting -> isTunnelInfoVisible = true
                is TunnelState.Connected -> isTunnelInfoVisible = true
                else -> isTunnelInfoVisible = false
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
    }

    private fun showTunnelInfo() {
        protocol.setText(R.string.wireguard)
    }
}
