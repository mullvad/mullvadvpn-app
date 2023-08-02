package net.mullvad.mullvadvpn.ui.widget

import androidx.annotation.DrawableRes

sealed class WidgetState {
    data class ImageState(@DrawableRes val imageRes: Int) : WidgetState()
    data class SwitchState(val isChecked: Boolean) : WidgetState()
}
