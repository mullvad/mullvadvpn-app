package net.mullvad.mullvadvpn.lib.endpoint

import android.os.Parcelable

interface ApiEndpointConfiguration : Parcelable {
    fun apiEndpoint(): ApiEndpoint?
}
