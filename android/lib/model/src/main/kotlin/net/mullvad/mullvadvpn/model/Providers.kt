package net.mullvad.mullvadvpn.model

import android.os.Parcelable
import kotlinx.parcelize.Parcelize

@Suppress("ensure value classes property is named value")
@JvmInline
@Parcelize
value class Providers(val providers: HashSet<String>) : Parcelable
