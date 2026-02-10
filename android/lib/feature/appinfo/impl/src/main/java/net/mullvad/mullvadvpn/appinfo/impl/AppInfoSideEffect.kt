package net.mullvad.mullvadvpn.appinfo.impl

import android.net.Uri

sealed interface AppInfoSideEffect {
    data class OpenUri(val uri: Uri, val errorMessage: String) : AppInfoSideEffect
}
