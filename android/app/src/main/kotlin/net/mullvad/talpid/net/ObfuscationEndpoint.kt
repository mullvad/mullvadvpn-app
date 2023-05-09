package net.mullvad.talpid.net

import android.os.Parcelable
import kotlinx.parcelize.Parcelize

@Parcelize
data class ObfuscationEndpoint(val endpoint: Endpoint, val obfuscationType: ObfuscationType) :
    Parcelable
