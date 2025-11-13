package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.WindowInsetsSides
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.only
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.systemBars
import androidx.compose.foundation.layout.windowInsetsPadding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyItemScope
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.tooling.preview.PreviewParameter
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.compose.dropUnlessResumed
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.annotation.RootGraph
import com.ramcosta.composedestinations.navigation.DestinationsNavigator
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.button.ApplyButton
import net.mullvad.mullvadvpn.compose.component.NavigateBackIconButton
import net.mullvad.mullvadvpn.compose.component.ScaffoldWithSmallTopBar
import net.mullvad.mullvadvpn.compose.constant.ContentType
import net.mullvad.mullvadvpn.compose.extensions.itemWithDivider
import net.mullvad.mullvadvpn.compose.extensions.itemsIndexedWithDivider
import net.mullvad.mullvadvpn.compose.extensions.itemsWithDivider
import net.mullvad.mullvadvpn.compose.preview.FilterUiStatePreviewParameterProvider
import net.mullvad.mullvadvpn.compose.state.RelayFilterUiState
import net.mullvad.mullvadvpn.compose.transitions.SlideInFromRightTransition
import net.mullvad.mullvadvpn.compose.util.CollectSideEffectWithLifecycle
import net.mullvad.mullvadvpn.lib.model.Constraint
import net.mullvad.mullvadvpn.lib.model.Ownership
import net.mullvad.mullvadvpn.lib.model.ProviderId
import net.mullvad.mullvadvpn.lib.model.Providers
import net.mullvad.mullvadvpn.lib.model.RelayListType
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.lib.ui.component.listitem.CheckableListItem
import net.mullvad.mullvadvpn.lib.ui.component.listitem.ExpandableListItem
import net.mullvad.mullvadvpn.lib.ui.component.listitem.SelectableListItem
import net.mullvad.mullvadvpn.lib.ui.designsystem.Hierarchy
import net.mullvad.mullvadvpn.lib.ui.designsystem.Position
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

data class FilterNavArgs(val relayListType: RelayListType)

@Destination<RootGraph>(style = SlideInFromRightTransition::class, navArgs = FilterNavArgs::class)
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
    onSelectedOwnership: (ownership: Constraint<Ownership>) -> Unit,
    onAllProviderCheckChange: (isChecked: Boolean) -> Unit,
    onSelectedProvider: (checked: Boolean, provider: ProviderId) -> Unit,
) {
    var providerExpanded by rememberSaveable { mutableStateOf(false) }
    var ownershipExpanded by rememberSaveable { mutableStateOf(false) }

    val backgroundColor = MaterialTheme.colorScheme.surface
    ScaffoldWithSmallTopBar(
        modifier = Modifier.background(backgroundColor),
        appBarTitle = stringResource(R.string.filter),
        navigationIcon = { NavigateBackIconButton(onNavigateBack = onBackClick) },
        bottomBar = {
            BottomBar(
                isApplyButtonEnabled = state.isApplyButtonEnabled,
                backgroundColor = backgroundColor,
                onApplyClick = onApplyClick,
            )
        },
    ) { modifier ->
        LazyColumn(modifier = modifier.fillMaxSize().padding(horizontal = Dimens.sideMarginNew)) {
            itemWithDivider(key = Keys.OWNERSHIP_TITLE, contentType = ContentType.HEADER) {
                OwnershipHeader(ownershipExpanded) { ownershipExpanded = it }
            }
            if (ownershipExpanded) {
                itemWithDivider(key = Keys.OWNERSHIP_ALL, contentType = ContentType.ITEM) {
                    AnyOwnership(state) { onSelectedOwnership(Constraint.Any) }
                }
                itemsIndexedWithDivider(
                    key = { _, item -> item.name },
                    contentType = { _, _ -> ContentType.ITEM },
                    items = state.selectableOwnerships,
                ) { index, ownership ->
                    val position =
                        if (index == state.selectableOwnerships.lastIndex) {
                            Position.Bottom
                        } else {
                            Position.Middle
                        }
                    Ownership(ownership, state, position) {
                        onSelectedOwnership(Constraint.Only(ownership))
                    }
                }
            }
            item(key = Keys.SPACER) {
                Spacer(modifier = Modifier.height(Dimens.cellVerticalSpacing).animateItem())
            }
            itemWithDivider(key = Keys.PROVIDERS_TITLE, contentType = ContentType.HEADER) {
                ProvidersHeader(providerExpanded) { providerExpanded = it }
            }
            if (providerExpanded) {
                itemWithDivider(key = Keys.PROVIDERS_ALL, contentType = ContentType.ITEM) {
                    AllProviders(state, onAllProviderCheckChange)
                }
                itemsWithDivider(
                    key = { it.value },
                    contentType = { ContentType.ITEM },
                    items = state.removedProviders,
                ) { provider ->
                    RemovedProvider(provider, state, onSelectedProvider)
                }

                itemsIndexedWithDivider(
                    key = { _, item -> item.value },
                    contentType = { _, _ -> ContentType.ITEM },
                    items = state.selectableProviders,
                ) { index, provider ->
                    val position =
                        if (index == state.selectableProviders.lastIndex) {
                            Position.Bottom
                        } else {
                            Position.Middle
                        }
                    Provider(provider, state, position, onSelectedProvider)
                }
            }
        }
    }
}

@Composable
private fun LazyItemScope.OwnershipHeader(expanded: Boolean, onToggleExpanded: (Boolean) -> Unit) {
    ExpandableListItem(
        title = stringResource(R.string.ownership),
        isExpanded = expanded,
        isEnabled = true,
        onInfoClicked = null,
        onCellClicked = { onToggleExpanded(!expanded) },
        modifier = Modifier.animateItem(),
        position =
            if (expanded) {
                Position.Top
            } else {
                Position.Single
            },
    )
}

@Composable
private fun LazyItemScope.AnyOwnership(state: RelayFilterUiState, onSelectedOwnership: () -> Unit) {
    SelectableListItem(
        title = stringResource(id = R.string.any),
        isSelected = state.selectedOwnership is Constraint.Any,
        modifier = Modifier.animateItem(),
        onClick = { onSelectedOwnership() },
        position = Position.Middle,
        hierarchy = Hierarchy.Child1,
    )
}

@Composable
private fun LazyItemScope.Ownership(
    ownership: Ownership,
    state: RelayFilterUiState,
    position: Position,
    onSelectedOwnership: (ownership: Ownership) -> Unit,
) {
    SelectableListItem(
        title = stringResource(id = ownership.stringResource()),
        isSelected = ownership == state.selectedOwnership.getOrNull(),
        modifier = Modifier.animateItem(),
        onClick = { onSelectedOwnership(ownership) },
        position = position,
        hierarchy = Hierarchy.Child1,
    )
}

@Composable
private fun LazyItemScope.ProvidersHeader(expanded: Boolean, onToggleExpanded: (Boolean) -> Unit) {
    ExpandableListItem(
        title = stringResource(R.string.providers),
        isExpanded = expanded,
        isEnabled = true,
        onInfoClicked = null,
        onCellClicked = { onToggleExpanded(!expanded) },
        modifier = Modifier.animateItem(),
        position =
            if (expanded) {
                Position.Top
            } else {
                Position.Single
            },
    )
}

@Composable
private fun LazyItemScope.AllProviders(
    state: RelayFilterUiState,
    onAllProviderCheckChange: (isChecked: Boolean) -> Unit,
) {
    val isChecked = state.isAllProvidersChecked
    CheckableListItem(
        title = stringResource(R.string.all_providers),
        isChecked = isChecked,
        onCheckedChange = { isChecked -> onAllProviderCheckChange(isChecked) },
        position = Position.Middle,
        hierarchy = Hierarchy.Child1,
        modifier = Modifier.animateItem(),
    )
}

@Composable
private fun LazyItemScope.Provider(
    providerId: ProviderId,
    state: RelayFilterUiState,
    position: Position,
    onSelectedProvider: (checked: Boolean, providerId: ProviderId) -> Unit,
) {
    val checked = providerId.isChecked(state.selectedProviders)
    CheckableListItem(
        title = providerId.value,
        isChecked = checked,
        onCheckedChange = { checked -> onSelectedProvider(checked, providerId) },
        modifier = Modifier.animateItem(),
        position = position,
        hierarchy = Hierarchy.Child1,
    )
}

private fun ProviderId.isChecked(constraint: Constraint<Providers>) =
    when (constraint) {
        Constraint.Any -> true
        is Constraint.Only -> this in constraint.value
    }

@Composable
private fun LazyItemScope.RemovedProvider(
    providerId: ProviderId,
    state: RelayFilterUiState,
    onSelectedProvider: (checked: Boolean, providerId: ProviderId) -> Unit,
) {
    val checked =
        state.selectedProviders is Constraint.Only && providerId in state.selectedProviders.value
    CheckableListItem(
        title = stringResource(R.string.removed_provider, providerId.value),
        isChecked = checked,
        isEnabled = checked,
        position = Position.Middle,
        onCheckedChange = { checked -> onSelectedProvider(checked, providerId) },
        modifier = Modifier.animateItem(),
        hierarchy = Hierarchy.Child1,
    )
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
                .windowInsetsPadding(WindowInsets.systemBars.only(WindowInsetsSides.Bottom))
                .padding(vertical = Dimens.screenBottomMargin, horizontal = Dimens.sideMargin),
        contentAlignment = Alignment.BottomCenter,
    ) {
        ApplyButton(onClick = onApplyClick, isEnabled = isApplyButtonEnabled)
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
    const val SPACER = "spacer"
}
