package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.systemBarsPadding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.Divider
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.tooling.preview.Preview
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.navigation.DestinationsNavigator
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.button.ApplyButton
import net.mullvad.mullvadvpn.compose.cell.CheckboxCell
import net.mullvad.mullvadvpn.compose.cell.ExpandableComposeCell
import net.mullvad.mullvadvpn.compose.cell.SelectableCell
import net.mullvad.mullvadvpn.compose.state.RelayFilterState
import net.mullvad.mullvadvpn.compose.transitions.SlideInFromRightTransition
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.model.Ownership
import net.mullvad.mullvadvpn.relaylist.Provider
import net.mullvad.mullvadvpn.viewmodel.FilterScreenSideEffect
import net.mullvad.mullvadvpn.viewmodel.FilterViewModel
import org.koin.androidx.compose.koinViewModel

@Preview
@Composable
private fun PreviewFilterScreen() {
    val state =
        RelayFilterState(
            selectedOwnership = null,
            allProviders = listOf(),
            selectedProviders = listOf(),
        )
    AppTheme {
        FilterScreen(
            uiState = state,
            onSelectedOwnership = {},
            onSelectedProvider = { _, _ -> },
            onAllProviderCheckChange = {},
        )
    }
}

@Destination(style = SlideInFromRightTransition::class)
@Composable
fun FilterScreen(navigator: DestinationsNavigator) {
    val viewModel = koinViewModel<FilterViewModel>()
    val uiState by viewModel.uiState.collectAsState()

    LaunchedEffect(Unit) {
        viewModel.uiSideEffect.collect {
            when (it) {
                FilterScreenSideEffect.CloseScreen -> navigator.navigateUp()
            }
        }
    }
    FilterScreen(
        uiState = uiState,
        onBackClick = navigator::navigateUp,
        onApplyClick = viewModel::onApplyButtonClicked,
        onSelectedOwnership = viewModel::setSelectedOwnership,
        onAllProviderCheckChange = viewModel::setAllProviders,
        onSelectedProvider = viewModel::setSelectedProvider
    )
}

@Composable
fun FilterScreen(
    uiState: RelayFilterState,
    onBackClick: () -> Unit = {},
    onApplyClick: () -> Unit = {},
    onSelectedOwnership: (ownership: Ownership?) -> Unit = {},
    onAllProviderCheckChange: (isChecked: Boolean) -> Unit = {},
    onSelectedProvider: (checked: Boolean, provider: Provider) -> Unit = { _, _ -> }
) {
    var providerExpanded by rememberSaveable { mutableStateOf(false) }
    var ownershipExpanded by rememberSaveable { mutableStateOf(false) }

    val backgroundColor = MaterialTheme.colorScheme.background

    Scaffold(
        modifier = Modifier.background(backgroundColor).systemBarsPadding().fillMaxSize(),
        topBar = {
            Row(Modifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically) {
                IconButton(onClick = onBackClick) {
                    Icon(
                        painter = painterResource(id = R.drawable.icon_back),
                        contentDescription = null,
                        tint = Color.Unspecified,
                    )
                }
                Text(
                    text = stringResource(R.string.filter),
                    modifier = Modifier.weight(1f).padding(end = Dimens.titleIconSize),
                    textAlign = TextAlign.Center,
                    style = MaterialTheme.typography.titleLarge,
                    color = MaterialTheme.colorScheme.onPrimary
                )
            }
        },
        bottomBar = {
            Box(
                modifier =
                    Modifier.fillMaxWidth()
                        .padding(top = Dimens.screenVerticalMargin)
                        .clickable(enabled = false, onClick = onApplyClick)
                        .background(color = backgroundColor),
                contentAlignment = Alignment.BottomCenter
            ) {
                ApplyButton(
                    onClick = onApplyClick,
                    isEnabled = uiState.isApplyButtonEnabled,
                    modifier =
                        Modifier.padding(
                            start = Dimens.sideMargin,
                            end = Dimens.sideMargin,
                            bottom = Dimens.screenVerticalMargin
                        ),
                )
            }
        },
    ) { contentPadding ->
        LazyColumn(modifier = Modifier.padding(contentPadding).fillMaxSize()) {
            item {
                Divider()
                ExpandableComposeCell(
                    title = stringResource(R.string.ownership),
                    isExpanded = ownershipExpanded,
                    isEnabled = true,
                    onInfoClicked = null,
                    onCellClicked = { ownershipExpanded = !ownershipExpanded }
                )
            }
            if (ownershipExpanded) {
                item {
                    SelectableCell(
                        title = stringResource(id = R.string.any),
                        isSelected = uiState.selectedOwnership == null,
                        onCellClicked = { onSelectedOwnership(null) }
                    )
                }
                items(uiState.filteredOwnershipByProviders) { ownership ->
                    Divider()
                    SelectableCell(
                        title = stringResource(id = ownership.stringResource()),
                        isSelected = ownership == uiState.selectedOwnership,
                        onCellClicked = { onSelectedOwnership(ownership) }
                    )
                }
            }
            item {
                Divider()
                ExpandableComposeCell(
                    title = stringResource(R.string.providers),
                    isExpanded = providerExpanded,
                    isEnabled = true,
                    onInfoClicked = null,
                    onCellClicked = { providerExpanded = !providerExpanded }
                )
            }
            if (providerExpanded) {
                item {
                    Divider()
                    CheckboxCell(
                        providerName = stringResource(R.string.all_providers),
                        checked = uiState.isAllProvidersChecked,
                        onCheckedChange = { isChecked -> onAllProviderCheckChange(isChecked) }
                    )
                }
                items(uiState.filteredProvidersByOwnership) { provider ->
                    Divider()
                    CheckboxCell(
                        providerName = provider.name,
                        checked = provider in uiState.selectedProviders,
                        onCheckedChange = { checked -> onSelectedProvider(checked, provider) }
                    )
                }
            }
        }
    }
}

private fun Ownership.stringResource(): Int =
    when (this) {
        Ownership.MullvadOwned -> R.string.mullvad_owned_only
        Ownership.Rented -> R.string.rented_only
    }
