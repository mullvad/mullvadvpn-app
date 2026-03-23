package net.mullvad.mullvadvpn.feature.vpnsettings.impl.util

import androidx.compose.ui.test.SemanticsNodeInteraction
import androidx.compose.ui.test.SemanticsNodeInteractionsProvider
import androidx.compose.ui.test.hasContentDescription
import androidx.compose.ui.test.hasParent
import androidx.compose.ui.test.hasTestTag

fun SemanticsNodeInteractionsProvider.onNodeWithContentDescriptionAndParentTag(
    contentDescription: String,
    parentTag: String,
    substring: Boolean = false,
    ignoreCase: Boolean = false,
    useUnmergedTree: Boolean = false,
): SemanticsNodeInteraction =
    onNode(
        hasContentDescription(contentDescription, substring, ignoreCase)
            .and(hasParent(hasTestTag(parentTag))),
        useUnmergedTree,
    )
