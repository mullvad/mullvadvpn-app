package net.mullvad.mullvadvpn.feature.location.impl

import androidx.compose.foundation.background
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Info
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import net.mullvad.mullvadvpn.lib.model.Ownership
import net.mullvad.mullvadvpn.lib.ui.component.text.FirstBaselineAlignedIconAndText
import net.mullvad.mullvadvpn.lib.ui.designsystem.MullvadFilterChip
import net.mullvad.mullvadvpn.lib.ui.theme.AppTheme
import net.mullvad.mullvadvpn.lib.ui.theme.Dimens
import net.mullvad.mullvadvpn.lib.usecase.FilterChip

@Preview
@Composable
private fun PreviewFilterActiveEntryContainer() {
    AppTheme {
        Column(modifier = Modifier.background(MaterialTheme.colorScheme.surface)) {
            FilterContainer(
                filters =
                    listOf(
                        FilterChip.Ownership(Ownership.MullvadOwned),
                        FilterChip.Provider(2),
                        FilterChip.Daita,
                    ),
                relayFiltersActive = false,
                onRemoveOwnershipFilter = {},
                onRemoveProviderFilter = {},
            )
        }
    }
}

@Preview
@Composable
private fun PreviewFilterInactiveEntryContainer() {
    AppTheme {
        Column(modifier = Modifier.background(MaterialTheme.colorScheme.surface)) {
            FilterContainer(
                filters =
                    listOf(
                        FilterChip.Ownership(Ownership.MullvadOwned),
                        FilterChip.Provider(2),
                        FilterChip.Daita,
                    ),
                relayFiltersActive = true,
                onRemoveOwnershipFilter = {},
                onRemoveProviderFilter = {},
            )
        }
    }
}

@Composable
fun FilterContainer(
    modifier: Modifier = Modifier,
    filters: List<FilterChip>,
    relayFiltersActive: Boolean,
    onRemoveOwnershipFilter: () -> Unit,
    onRemoveProviderFilter: () -> Unit,
) {
    Column(modifier = modifier) {
        FilterRow(
            filters = filters,
            relayFiltersActive = relayFiltersActive,
            onRemoveOwnershipFilter = onRemoveOwnershipFilter,
            onRemoveProviderFilter = onRemoveProviderFilter,
        )

        val showInactiveFiltersInfo = !relayFiltersActive && filters.any { it.type == FilterChip.Type.Relay }
        if (showInactiveFiltersInfo) {
            FirstBaselineAlignedIconAndText(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(horizontal = Dimens.mediumSpacer),
                text = "Filters are overridden when using an automatic location",
                icon = Icons.Rounded.Info,
                iconSize = Dimens.smallIconSize,
                iconTint = MaterialTheme.colorScheme.onSurface,
                textColor = MaterialTheme.colorScheme.onSurfaceVariant,
                textStyle = MaterialTheme.typography.bodyMedium,
            )
        }
    }
}


@Composable
fun FilterRow(
    filters: List<FilterChip>,
    modifier: Modifier = Modifier,
    relayFiltersActive: Boolean,
    onRemoveOwnershipFilter: () -> Unit,
    onRemoveProviderFilter: () -> Unit,
) {
    val scrollState = rememberScrollState()
    Row(
        verticalAlignment = Alignment.CenterVertically,
        modifier = modifier
            .fillMaxWidth()
            .horizontalScroll(scrollState),
        horizontalArrangement = Arrangement.spacedBy(Dimens.chipSpace),
    ) {
        Spacer(modifier = Modifier.width(Dimens.smallPadding))
        filters.forEach {
            when (it) {
                is FilterChip.Ownership ->
                    OwnershipFilterChip(
                        ownership = it.ownership,
                        active = relayFiltersActive,
                        onRemoveClick = onRemoveOwnershipFilter,
                    )
                is FilterChip.Provider ->
                    ProviderFilterChip(
                        providers = it.count,
                        active = relayFiltersActive,
                        onRemoveClick = onRemoveProviderFilter,
                    )
                is FilterChip.Daita -> DaitaFilterChip()
                is FilterChip.Entry -> EntryFilterChip()
                is FilterChip.Exit -> ExitFilterChip()
                is FilterChip.Quic -> QuicFilterChip()
                is FilterChip.Lwo -> LwoFilterChip()
            }
        }
        Spacer(modifier = Modifier.width(Dimens.smallPadding))
    }
}

@Composable
private fun ProviderFilterChip(providers: Int, onRemoveClick: () -> Unit, active: Boolean) {
    MullvadFilterChip(
        text = stringResource(id = R.string.number_of_providers, providers),
        onRemoveClick = onRemoveClick,
        enabled = true,
        active = active,
    )
}

@Composable
private fun OwnershipFilterChip(ownership: Ownership, onRemoveClick: () -> Unit, active: Boolean) {
    MullvadFilterChip(
        text = stringResource(ownership.stringResources()),
        onRemoveClick = onRemoveClick,
        enabled = true,
        active = active,
    )
}

@Composable
private fun DaitaFilterChip() {
    MullvadFilterChip(
        text = stringResource(id = R.string.daita),
        onRemoveClick = {},
        enabled = false,
    )
}

@Composable
private fun EntryFilterChip() {
    MullvadFilterChip(
        text = stringResource(id = R.string.entry),
        onRemoveClick = {},
        enabled = false,
    )
}

@Composable
private fun ExitFilterChip() {
    MullvadFilterChip(
        text = stringResource(id = R.string.exit),
        onRemoveClick = {},
        enabled = false,
    )
}

@Composable
private fun QuicFilterChip() {
    MullvadFilterChip(
        text = stringResource(id = R.string.quic),
        onRemoveClick = {},
        enabled = false,
    )
}

@Composable
private fun LwoFilterChip() {
    MullvadFilterChip(text = stringResource(id = R.string.lwo), onRemoveClick = {}, enabled = false)
}

private fun Ownership.stringResources(): Int =
    when (this) {
        Ownership.MullvadOwned -> R.string.owned
        Ownership.Rented -> R.string.rented
    }
