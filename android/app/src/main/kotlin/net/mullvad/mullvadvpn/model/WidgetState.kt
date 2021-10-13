package net.mullvad.mullvadvpn.model

import androidx.annotation.DrawableRes

sealed class WidgetState {
    data class ImageState(@DrawableRes val imageRes: Int) : WidgetState()
    data class SwitchState(val isChecked: Boolean) : WidgetState()
}
