package net.mullvad.mullvadvpn.feature.anticensorship.impl.customport

import net.mullvad.mullvadvpn.lib.model.Port

sealed interface CustomPortDialogSideEffect {
    data class Success(val port: Port?) : CustomPortDialogSideEffect
}
