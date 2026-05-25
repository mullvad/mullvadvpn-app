package net.mullvad.mullvadvpn.feature.lansharing.impl

import androidx.compose.animation.AnimatedVisibilityScope
import androidx.compose.animation.SharedTransitionScope
import androidx.compose.foundation.Image
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Info
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.tooling.preview.PreviewParameter
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.compose.dropUnlessResumed
import kotlin.text.appendLine
import net.mullvad.mullvadvpn.common.compose.unlessIsDetail
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.lib.common.Lc
import net.mullvad.mullvadvpn.lib.model.FeatureIndicator
import net.mullvad.mullvadvpn.lib.ui.component.Accordion
import net.mullvad.mullvadvpn.lib.ui.component.ScaffoldWithSmallTopBar
import net.mullvad.mullvadvpn.lib.ui.component.annotatedStringResource
import net.mullvad.mullvadvpn.lib.ui.component.button.NavigateBackIconButton
import net.mullvad.mullvadvpn.lib.ui.component.button.NavigateCloseIconButton
import net.mullvad.mullvadvpn.lib.ui.component.drawVerticalScrollbar
import net.mullvad.mullvadvpn.lib.ui.component.listitem.SwitchListItem
import net.mullvad.mullvadvpn.lib.ui.component.text.ScreenDescription
import net.mullvad.mullvadvpn.lib.ui.designsystem.MullvadCircularProgressIndicatorLarge
import net.mullvad.mullvadvpn.lib.ui.tag.LOCAL_NETWORK_SHARING_SCREEN_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.theme.AppTheme
import net.mullvad.mullvadvpn.lib.ui.theme.Dimens
import net.mullvad.mullvadvpn.lib.ui.theme.color.AlphaScrollbar
import org.koin.androidx.compose.koinViewModel
import org.koin.core.parameter.parametersOf

@Preview("Loading|Disabled|Enabled")
@Composable
private fun PreviewLocalNetworkSharingScreen(
    @PreviewParameter(LocalNetworkSharingUiStatePreviewParameterProvider::class)
    state: Lc<Boolean, LocalNetworkSharingUiState>
) {
    AppTheme {
        LocalNetworkSharingScreen(
            state = state,
            onLocalNetworkSharingEnable = { _ -> },
            onBackClick = {},
        )
    }
}

@Composable
fun SharedTransitionScope.LocalNetworkSharing(
    navigator: Navigator,
    isModal: Boolean,
    animatedVisibilityScope: AnimatedVisibilityScope,
) {
    val viewModel = koinViewModel<LocalNetworkSharingViewModel> { parametersOf(isModal) }
    val state by viewModel.uiState.collectAsStateWithLifecycle()

    LocalNetworkSharingScreen(
        state = state,
        modifier =
            Modifier.testTag(LOCAL_NETWORK_SHARING_SCREEN_TEST_TAG)
                .sharedBounds(
                    rememberSharedContentState(key = FeatureIndicator.LAN_SHARING),
                    animatedVisibilityScope = animatedVisibilityScope,
                ),
        onLocalNetworkSharingEnable = viewModel::setLocalNetworkSharingEnabled,
        onBackClick = dropUnlessResumed { navigator.goBack() },
    )
}

@Composable
fun LocalNetworkSharingScreen(
    state: Lc<Boolean, LocalNetworkSharingUiState>,
    onLocalNetworkSharingEnable: (enable: Boolean) -> Unit,
    onBackClick: () -> Unit,
    modifier: Modifier = Modifier,
) {
    ScaffoldWithSmallTopBar(
        appBarTitle = stringResource(id = R.string.local_network_sharing),
        modifier = modifier,
        navigationIcon = {
            if (state.isModal()) {
                NavigateCloseIconButton { onBackClick() }
            } else {
                unlessIsDetail { NavigateBackIconButton { onBackClick() } }
            }
        },
    ) { modifier ->
        val scrollState = rememberScrollState()
        Column(
            horizontalAlignment = Alignment.CenterHorizontally,
            modifier =
                modifier
                    .drawVerticalScrollbar(
                        state = scrollState,
                        color = MaterialTheme.colorScheme.onSurface.copy(alpha = AlphaScrollbar),
                    )
                    .verticalScroll(state = scrollState),
        ) {
            when (state) {
                is Lc.Loading -> {
                    Loading()
                }
                is Lc.Content -> {
                    LocalNetworkSharingContent(
                        state = state.value,
                        onLanSharingEnabled = onLocalNetworkSharingEnable,
                    )
                }
            }
        }
    }
}

@Composable
private fun LocalNetworkSharingContent(
    state: LocalNetworkSharingUiState,
    onLanSharingEnabled: (enable: Boolean) -> Unit,
) {
    Column(modifier = Modifier.fillMaxWidth().padding(horizontal = Dimens.sideMarginNew)) {
        Image(
            contentScale = ContentScale.FillWidth,
            modifier =
                Modifier.widthIn(max = Dimens.settingsDetailsImageMaxWidth)
                    .fillMaxWidth()
                    .align(Alignment.CenterHorizontally),
            painter = painterResource(id = R.drawable.local_network_sharing_illustration),
            contentDescription = stringResource(R.string.local_network_sharing),
        )

        ScreenDescription(
            modifier = Modifier.padding(vertical = Dimens.mediumPadding),
            text =
                buildAnnotatedString {
                    appendLine(annotatedStringResource(R.string.local_network_sharing_info))
                    appendLine()
                    append(annotatedStringResource(R.string.local_network_sharing_info2))
                },
        )

        var expandedState by rememberSaveable { mutableStateOf(false) }

        Accordion(
            title = stringResource(R.string.how_it_works),
            expandedText =
                buildAnnotatedString {
                    appendLine(annotatedStringResource(R.string.local_network_sharing_info3))
                    appendLine()
                    append(annotatedStringResource(R.string.local_network_sharing_info4))
                },
            icon = Icons.Rounded.Info,
            iconContentDescription = stringResource(R.string.info),
            isExpanded = expandedState,
            onClick = { expandedState = !expandedState },
        )

        SwitchListItem(
            modifier = Modifier.padding(vertical = Dimens.screenBottomMarginNew),
            title = stringResource(R.string.enable),
            isToggled = state.lanSharingEnabled,
            onCellClicked = onLanSharingEnabled,
        )
    }
}

@Composable
private fun Loading() {
    MullvadCircularProgressIndicatorLarge()
}

private fun Lc<Boolean, LocalNetworkSharingUiState>.isModal() =
    when (this) {
        is Lc.Loading -> this.value
        is Lc.Content -> this.value.isModal
    }
