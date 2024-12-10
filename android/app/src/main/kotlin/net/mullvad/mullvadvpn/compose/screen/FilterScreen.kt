package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.systemBarsPadding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyItemScope
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.tooling.preview.PreviewParameter
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.compose.dropUnlessResumed
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.annotation.RootGraph
import com.ramcosta.composedestinations.navigation.DestinationsNavigator
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.button.ApplyButton
import net.mullvad.mullvadvpn.compose.cell.CheckboxCell
import net.mullvad.mullvadvpn.compose.cell.ExpandableComposeCell
import net.mullvad.mullvadvpn.compose.cell.SelectableCell
import net.mullvad.mullvadvpn.compose.constant.ContentType
import net.mullvad.mullvadvpn.compose.extensions.itemWithDivider
import net.mullvad.mullvadvpn.compose.extensions.itemsWithDivider
import net.mullvad.mullvadvpn.compose.preview.FilterUiStatePreviewParameterProvider
import net.mullvad.mullvadvpn.compose.state.RelayFilterUiState
import net.mullvad.mullvadvpn.compose.transitions.SlideInFromRightTransition
import net.mullvad.mullvadvpn.compose.util.CollectSideEffectWithLifecycle
import net.mullvad.mullvadvpn.lib.model.Ownership
import net.mullvad.mullvadvpn.lib.model.Provider
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.viewmodel.FilterScreenSideEffect
import net.mullvad.mullvadvpn.viewmodel.FilterViewModel
import org.koin.androidx.compose.koinViewModel

@Preview
@Composable
private fun PreviewFilterScreen(
    @PreviewParameter(FilterUiStatePreviewParameterProvider::class) state: RelayFilterUiState
) {
    AppTheme {
        FilterScreen(
            state = state,
            onSelectedOwnership = {},
            onSelectedProvider = { _, _ -> },
            onAllProviderCheckChange = {},
            onBackClick = {},
            onApplyClick = {},
        )
    }
}

@Destination<RootGraph>(style = SlideInFromRightTransition::class)
@Composable
fun Filter(navigator: DestinationsNavigator) {
    val viewModel = koinViewModel<FilterViewModel>()
    val state by viewModel.uiState.collectAsStateWithLifecycle()

    CollectSideEffectWithLifecycle(viewModel.uiSideEffect) {
        when (it) {
            FilterScreenSideEffect.CloseScreen -> navigator.navigateUp()
        }
    }
    FilterScreen(
        state = state,
        onBackClick = dropUnlessResumed { navigator.navigateUp() },
        onApplyClick = viewModel::onApplyButtonClicked,
        onSelectedOwnership = viewModel::setSelectedOwnership,
        onAllProviderCheckChange = viewModel::setAllProviders,
        onSelectedProvider = viewModel::setSelectedProvider,
    )
}

@Composable
fun FilterScreen(
    state: RelayFilterUiState,
    onBackClick: () -> Unit,
    onApplyClick: () -> Unit,
    onSelectedOwnership: (ownership: Ownership?) -> Unit,
    onAllProviderCheckChange: (isChecked: Boolean) -> Unit,
    onSelectedProvider: (checked: Boolean, provider: Provider) -> Unit,
) {
    var providerExpanded by rememberSaveable { mutableStateOf(false) }
    var ownershipExpanded by rememberSaveable { mutableStateOf(false) }

    val backgroundColor = MaterialTheme.colorScheme.surface
    Scaffold(
        modifier = Modifier.background(backgroundColor).systemBarsPadding().fillMaxSize(),
        topBar = { TopBar(onBackClick = onBackClick) },
        bottomBar = {
            BottomBar(
                isApplyButtonEnabled = state.isApplyButtonEnabled,
                backgroundColor = backgroundColor,
                onApplyClick = onApplyClick,
            )
        },
    ) { contentPadding ->
        LazyColumn(modifier = Modifier.padding(contentPadding).fillMaxSize()) {
            itemWithDivider(key = Keys.OWNERSHIP_TITLE, contentType = ContentType.HEADER) {
                OwnershipHeader(ownershipExpanded) { ownershipExpanded = it }
            }
            if (ownershipExpanded) {
                item(key = Keys.OWNERSHIP_ALL, contentType = ContentType.ITEM) {
                    AnyOwnership(state, onSelectedOwnership)
                }
                itemsWithDivider(
                    key = { it.name },
                    contentType = { ContentType.ITEM },
                    items = state.filteredOwnershipByProviders,
                ) { ownership ->
                    Ownership(ownership, state, onSelectedOwnership)
                }
            }
            itemWithDivider(key = Keys.PROVIDERS_TITLE, contentType = ContentType.HEADER) {
                ProvidersHeader(providerExpanded) { providerExpanded = it }
            }
            if (providerExpanded) {
                itemWithDivider(key = Keys.PROVIDERS_ALL, contentType = ContentType.ITEM) {
                    AllProviders(state, onAllProviderCheckChange)
                }
                itemsWithDivider(
                    key = { it.providerId.value },
                    contentType = { ContentType.ITEM },
                    items = state.filteredProvidersByOwnership,
                ) { provider ->
                    Provider(provider, state, onSelectedProvider)
                }
            }
        }
    }
}

@Composable
private fun LazyItemScope.OwnershipHeader(expanded: Boolean, onToggleExpanded: (Boolean) -> Unit) {
    ExpandableComposeCell(
        title = stringResource(R.string.ownership),
        isExpanded = expanded,
        isEnabled = true,
        onInfoClicked = null,
        onCellClicked = { onToggleExpanded(!expanded) },
        modifier = Modifier.animateItem(),
    )
}

@Composable
private fun LazyItemScope.AnyOwnership(
    state: RelayFilterUiState,
    onSelectedOwnership: (ownership: Ownership?) -> Unit,
) {
    SelectableCell(
        title = stringResource(id = R.string.any),
        isSelected = state.selectedOwnership == null,
        onCellClicked = { onSelectedOwnership(null) },
        modifier = Modifier.animateItem(),
    )
}

@Composable
private fun LazyItemScope.Ownership(
    ownership: Ownership,
    state: RelayFilterUiState,
    onSelectedOwnership: (ownership: Ownership?) -> Unit,
) {
    SelectableCell(
        title = stringResource(id = ownership.stringResource()),
        isSelected = ownership == state.selectedOwnership,
        onCellClicked = { onSelectedOwnership(ownership) },
        modifier = Modifier.animateItem(),
    )
}

@Composable
private fun LazyItemScope.ProvidersHeader(expanded: Boolean, onToggleExpanded: (Boolean) -> Unit) {
    ExpandableComposeCell(
        title = stringResource(R.string.providers),
        isExpanded = expanded,
        isEnabled = true,
        onInfoClicked = null,
        onCellClicked = { onToggleExpanded(!expanded) },
        modifier = Modifier.animateItem(),
    )
}

@Composable
private fun LazyItemScope.AllProviders(
    state: RelayFilterUiState,
    onAllProviderCheckChange: (isChecked: Boolean) -> Unit,
) {
    CheckboxCell(
        title = stringResource(R.string.all_providers),
        checked = state.isAllProvidersChecked,
        onCheckedChange = { isChecked -> onAllProviderCheckChange(isChecked) },
        modifier = Modifier.animateItem(),
    )
}

@Composable
private fun LazyItemScope.Provider(
    provider: Provider,
    state: RelayFilterUiState,
    onSelectedProvider: (checked: Boolean, provider: Provider) -> Unit,
) {
    CheckboxCell(
        title = provider.providerId.value,
        checked = provider in state.selectedProviders,
        onCheckedChange = { checked -> onSelectedProvider(checked, provider) },
        modifier = Modifier.animateItem(),
    )
}

@Composable
private fun TopBar(onBackClick: () -> Unit) {
    Row(Modifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically) {
        IconButton(onClick = onBackClick) {
            Icon(
                imageVector = Icons.AutoMirrored.Default.ArrowBack,
                contentDescription = stringResource(id = R.string.back),
                tint = MaterialTheme.colorScheme.onSurface,
            )
        }
        Text(
            text = stringResource(R.string.filter),
            modifier = Modifier.weight(1f).padding(end = Dimens.titleIconSize),
            textAlign = TextAlign.Center,
            style = MaterialTheme.typography.titleLarge,
            color = MaterialTheme.colorScheme.onSurface,
        )
    }
}

@Composable
private fun BottomBar(
    isApplyButtonEnabled: Boolean,
    backgroundColor: Color,
    onApplyClick: () -> Unit,
) {
    Box(
        modifier =
            Modifier.fillMaxWidth()
                .background(color = backgroundColor)
                .padding(top = Dimens.screenVerticalMargin),
        contentAlignment = Alignment.BottomCenter,
    ) {
        ApplyButton(
            onClick = onApplyClick,
            isEnabled = isApplyButtonEnabled,
            modifier =
                Modifier.padding(
                    start = Dimens.sideMargin,
                    end = Dimens.sideMargin,
                    bottom = Dimens.screenVerticalMargin,
                ),
        )
    }
}

private fun Ownership.stringResource(): Int =
    when (this) {
        Ownership.MullvadOwned -> R.string.mullvad_owned_only
        Ownership.Rented -> R.string.rented_only
    }

private object Keys {
    const val OWNERSHIP_TITLE = "ownership_title"
    const val OWNERSHIP_ALL = "ownership_all"
    const val PROVIDERS_TITLE = "providers_title"
    const val PROVIDERS_ALL = "providers_all"
}
