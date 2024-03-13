package net.mullvad.mullvadvpn

import androidx.compose.ui.test.ExperimentalTestApi
import androidx.compose.ui.test.SemanticsNodeInteraction
import androidx.compose.ui.test.invokeGlobalAssertions
import androidx.compose.ui.test.longClick
import androidx.compose.ui.test.performTouchInput

fun SemanticsNodeInteraction.performLongClick(): SemanticsNodeInteraction {
    @OptIn(ExperimentalTestApi::class) return this.invokeGlobalAssertions().performLongClickImpl()
}

private fun SemanticsNodeInteraction.performLongClickImpl(): SemanticsNodeInteraction {
    return performTouchInput { longClick() }
}
