package net.mullvad.mullvadvpn.model

import android.os.Parcelable
import arrow.optics.optics
import kotlinx.parcelize.Parcelize

@Parcelize
@optics
data class DnsOptions(
    val currentDnsOption: DnsState,
    val defaultOptions: DefaultDnsOptions,
    val customOptions: CustomDnsOptions
) : Parcelable {
    companion object
}
