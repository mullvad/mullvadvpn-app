package net.mullvad.mullvadvpn

import androidx.compose.ui.test.SemanticsNodeInteraction
import androidx.compose.ui.test.longClick
import androidx.compose.ui.test.performTouchInput

fun SemanticsNodeInteraction.performLongClick(): SemanticsNodeInteraction {
    return this.performTouchInput { longClick() }
}
