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
import net.mullvad.mullvadvpn.lib.model.Port
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
                        FilterChip.Shadowsocks(Port(50_000)),
                    ),
                relayFiltersActive = false,
                onRemoveOwnershipFilter = {},
                onRemoveProviderFilter = {},
                onFilterNavigate = {},
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
                onFilterNavigate = {},
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
    onFilterNavigate: (FilterChip) -> Unit,
) {
    Column(modifier = modifier) {
        FilterRow(
            filters = filters,
            relayFiltersActive = relayFiltersActive,
            onRemoveOwnershipFilter = onRemoveOwnershipFilter,
            onRemoveProviderFilter = onRemoveProviderFilter,
            onFilterChipNavigate = onFilterNavigate,
        )

        val showInactiveFiltersInfo =
            !relayFiltersActive && filters.any { it.type == FilterChip.Type.Relay }
        if (showInactiveFiltersInfo) {
            FirstBaselineAlignedIconAndText(
                modifier = Modifier.fillMaxWidth().padding(horizontal = Dimens.mediumSpacer),
                text = stringResource(R.string.entry_filters_overridden_info),
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
    onFilterChipNavigate: (FilterChip) -> Unit,
) {
    val scrollState = rememberScrollState()
    Row(
        verticalAlignment = Alignment.CenterVertically,
        modifier = modifier.fillMaxWidth().horizontalScroll(scrollState),
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
                is FilterChip.Shadowsocks ->
                    ShadowsocksFilterChip(
                        port = it.port,
                        onClick = { onFilterChipNavigate(it) },
                    )
                is FilterChip.Daita -> DaitaFilterChip { onFilterChipNavigate(it) }
                is FilterChip.Quic -> QuicFilterChip { onFilterChipNavigate(it) }
                is FilterChip.Lwo -> LwoFilterChip { onFilterChipNavigate(it) }
                is FilterChip.Entry -> EntryFilterChip()
                is FilterChip.Exit -> ExitFilterChip()
            }
        }
        Spacer(modifier = Modifier.width(Dimens.smallPadding))
    }
}

@Composable
private fun ProviderFilterChip(providers: Int, onRemoveClick: () -> Unit, active: Boolean) {
    MullvadFilterChip(
        text = stringResource(id = R.string.number_of_providers, providers),
        onClick = onRemoveClick,
        enabled = true,
        showCrossIcon = true,
        active = active,
    )
}

@Composable
private fun OwnershipFilterChip(ownership: Ownership, onRemoveClick: () -> Unit, active: Boolean) {
    MullvadFilterChip(
        text = stringResource(ownership.stringResources()),
        onClick = onRemoveClick,
        enabled = true,
        showCrossIcon = true,
        active = active,
    )
}

@Composable
private fun ShadowsocksFilterChip(port: Port, onClick: () -> Unit) {
    MullvadFilterChip(
        text = stringResource(R.string.shadowsocks_port, port.value),
        onClick = onClick,
        enabled = true,
    )
}

@Composable
private fun DaitaFilterChip(onClick: () -> Unit) {
    MullvadFilterChip(
        text = stringResource(id = R.string.daita),
        onClick = onClick,
    )
}

@Composable
private fun EntryFilterChip() {
    MullvadFilterChip(
        text = stringResource(id = R.string.entry),
        onClick = {},
        enabled = false,
    )
}

@Composable
private fun ExitFilterChip() {
    MullvadFilterChip(
        text = stringResource(id = R.string.exit),
        onClick = {},
        enabled = false,
    )
}

@Composable
private fun QuicFilterChip(onClick: () -> Unit) {
    MullvadFilterChip(
        text = stringResource(id = R.string.quic),
        onClick = onClick,
    )
}

@Composable
private fun LwoFilterChip(onClick: () -> Unit) {
    MullvadFilterChip(text = stringResource(id = R.string.lwo), onClick = onClick)
}

private fun Ownership.stringResources(): Int =
    when (this) {
        Ownership.MullvadOwned -> R.string.owned
        Ownership.Rented -> R.string.rented
    }
