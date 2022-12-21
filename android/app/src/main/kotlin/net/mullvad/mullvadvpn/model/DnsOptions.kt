package net.mullvad.mullvadvpn.model

import android.os.Parcelable
import kotlinx.parcelize.Parcelize

@Parcelize
data class DnsOptions(
    val state: DnsState,
    val defaultOptions: DefaultDnsOptions,
    val customOptions: CustomDnsOptions
) : Parcelable
