package net.mullvad.mullvadvpn

import androidx.compose.ui.test.SemanticsNodeInteraction
import androidx.compose.ui.test.SemanticsNodeInteractionsProvider
import androidx.compose.ui.test.hasTestTag
import androidx.compose.ui.test.hasText

fun SemanticsNodeInteractionsProvider.onNodeWithTagAndText(
    testTag: String,
    text: String,
    substring: Boolean = false,
    ignoreCase: Boolean = false,
    useUnmergedTree: Boolean = false
): SemanticsNodeInteraction =
    onNode(hasTestTag(testTag).and(hasText(text, substring, ignoreCase)), useUnmergedTree)
