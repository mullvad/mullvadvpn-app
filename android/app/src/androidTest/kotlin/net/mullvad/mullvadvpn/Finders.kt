package net.mullvad.mullvadvpn

import androidx.compose.ui.test.SemanticsNodeInteraction
import androidx.compose.ui.test.SemanticsNodeInteractionsProvider
import androidx.compose.ui.test.hasContentDescription
import androidx.compose.ui.test.hasParent
import androidx.compose.ui.test.hasTestTag
import androidx.compose.ui.test.hasText

fun SemanticsNodeInteractionsProvider.onNodeWithTagAndText(
    testTag: String,
    text: String,
    substring: Boolean = false,
    ignoreCase: Boolean = false,
    useUnmergedTree: Boolean = false,
): SemanticsNodeInteraction =
    onNode(hasTestTag(testTag).and(hasText(text, substring, ignoreCase)), useUnmergedTree)

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
