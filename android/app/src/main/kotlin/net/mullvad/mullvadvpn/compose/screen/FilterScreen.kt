package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.Divider
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.tooling.preview.Preview
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.SharedFlow
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.button.ApplyButton
import net.mullvad.mullvadvpn.compose.cell.CheckboxCell
import net.mullvad.mullvadvpn.compose.cell.ExpandableComposeCell
import net.mullvad.mullvadvpn.compose.cell.SelectableCell
import net.mullvad.mullvadvpn.compose.state.RelayFilterState
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.model.Ownership
import net.mullvad.mullvadvpn.relaylist.Provider

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
            onSelectedProviders = { _, _ -> },
            onAllProviderCheckChange = {},
            uiCloseAction = MutableSharedFlow()
        )
    }
}

@Composable
fun FilterScreen(
    uiState: RelayFilterState,
    onBackClick: () -> Unit = {},
    uiCloseAction: SharedFlow<Unit>,
    onApplyClick: () -> Unit = {},
    onSelectedOwnership: (ownership: Ownership?) -> Unit = {},
    onAllProviderCheckChange: (isChecked: Boolean) -> Unit = {},
    onSelectedProviders: (checked: Boolean, provider: Provider) -> Unit
) {
    var providerExpanded by rememberSaveable { mutableStateOf(false) }
    var ownershipExpanded by rememberSaveable { mutableStateOf(false) }

    val backgroundColor = MaterialTheme.colorScheme.background

    LaunchedEffect(Unit) { uiCloseAction.collect { onBackClick() } }
    Scaffold(
        topBar = {
            Row(
                Modifier.padding(
                        horizontal = Dimens.selectFilterTitlePadding,
                        vertical = Dimens.selectFilterTitlePadding
                    )
                    .fillMaxWidth(),
            ) {
                Image(
                    painter = painterResource(id = R.drawable.icon_back),
                    contentDescription = null,
                    modifier = Modifier.size(Dimens.titleIconSize).clickable(onClick = onBackClick)
                )
                Text(
                    text = stringResource(R.string.filter),
                    modifier =
                        Modifier.align(Alignment.CenterVertically)
                            .weight(weight = 1f)
                            .padding(end = Dimens.titleIconSize),
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
        LazyColumn(
            modifier = Modifier.padding(contentPadding).background(backgroundColor).fillMaxSize()
        ) {
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
                        onCheckedChange = { checked -> onSelectedProviders(checked, provider) }
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
