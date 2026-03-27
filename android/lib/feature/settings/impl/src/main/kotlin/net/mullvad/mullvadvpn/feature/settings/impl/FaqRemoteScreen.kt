package net.mullvad.mullvadvpn.feature.settings.impl

import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.remote.player.compose.RemoteDocumentPlayer
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.res.stringResource
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.compose.dropUnlessResumed
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.lib.common.Lc
import net.mullvad.mullvadvpn.lib.ui.component.NavigateBackIconButton
import net.mullvad.mullvadvpn.lib.ui.component.ScaffoldWithSmallTopBar
import net.mullvad.mullvadvpn.lib.ui.designsystem.MullvadCircularProgressIndicatorLarge
import org.koin.androidx.compose.koinViewModel

@Composable
fun FaqRemote(navigator: Navigator) {
    val viewmodel = koinViewModel<FaqRemoteViewModel>()
    val state by viewmodel.uiState.collectAsStateWithLifecycle()
    FaqRemoteScreen(state = state, onBackClick = dropUnlessResumed { navigator.goBack() })
}

@Composable
private fun FaqRemoteScreen(state: Lc<Unit, FaqRemoteState>, onBackClick: () -> Unit) {
    ScaffoldWithSmallTopBar(
        appBarTitle = stringResource(R.string.faqs_and_guides),
        navigationIcon = { NavigateBackIconButton(onNavigateBack = onBackClick) },
    ) { modifier ->
        BoxWithConstraints(modifier = modifier) {
            val screenWidth = constraints.maxWidth
            val screenHeight = constraints.maxHeight

            when (state) {
                is Lc.Loading -> {
                    MullvadCircularProgressIndicatorLarge()
                }
                is Lc.Content -> {
                    RemoteDocumentPlayer(
                        document = state.value.document,
                        documentWidth = screenWidth,
                        documentHeight = screenHeight,
                    )
                }
            }
        }
    }
}
