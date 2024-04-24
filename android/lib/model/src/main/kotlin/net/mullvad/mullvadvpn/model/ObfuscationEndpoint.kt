package net.mullvad.mullvadvpn.model

import android.os.Parcelable
import kotlinx.parcelize.Parcelize

@Parcelize
data class ObfuscationEndpoint(val endpoint: Endpoint, val obfuscationType: ObfuscationType) :
    Parcelable
