package de.mannodermaus.junit5.compose

import androidx.compose.runtime.Composable
import androidx.compose.ui.test.SemanticsNodeInteractionsProvider

interface ComposeContext : SemanticsNodeInteractionsProvider {
    fun setContent(content: @Composable () -> Unit)

    fun waitForIdle()
}
