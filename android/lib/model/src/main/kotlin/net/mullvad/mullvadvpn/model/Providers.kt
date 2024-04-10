package net.mullvad.mullvadvpn.model

import android.os.Parcelable
import kotlinx.parcelize.Parcelize

@Parcelize data class Providers(val providers: Set<ProviderId>) : Parcelable
