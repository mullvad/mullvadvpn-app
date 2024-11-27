package net.mullvad.mullvadvpn.compose.cell

import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.component.MullvadFilterChip
import net.mullvad.mullvadvpn.lib.model.Ownership
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.usecase.FilterChip

@Preview
@Composable
private fun PreviewFilterCell() {
    AppTheme {
        FilterRow(
            filters = listOf(FilterChip.Ownership(Ownership.MullvadOwned), FilterChip.Provider(2)),
            onRemoveOwnershipFilter = {},
            onRemoveProviderFilter = {},
        )
    }
}

@Composable
fun FilterRow(
    filters: List<FilterChip>,
    showTitle: Boolean = true,
    onRemoveOwnershipFilter: () -> Unit,
    onRemoveProviderFilter: () -> Unit,
) {
    val scrollState = rememberScrollState()
    Row(
        verticalAlignment = Alignment.CenterVertically,
        modifier =
            Modifier.padding(horizontal = Dimens.searchFieldHorizontalPadding)
                .fillMaxWidth()
                .horizontalScroll(scrollState),
        horizontalArrangement = Arrangement.spacedBy(Dimens.chipSpace),
    ) {
        if (showTitle) {
            Text(
                text = stringResource(id = R.string.filters),
                color = MaterialTheme.colorScheme.onPrimary,
                style = MaterialTheme.typography.labelMedium,
            )
        }
        filters.forEach {
            when (it) {
                is FilterChip.Ownership ->
                    OwnershipFilterChip(it.ownership, onRemoveOwnershipFilter)
                is FilterChip.Provider -> ProviderFilterChip(it.count, onRemoveProviderFilter)
                is FilterChip.Daita -> DaitaFilterChip()
                is FilterChip.Entry -> EntryFilterChip()
                is FilterChip.Exit -> ExitFilterChip()
            }
        }
    }
}

@Composable
fun ProviderFilterChip(providers: Int, onRemoveClick: () -> Unit) {
    MullvadFilterChip(
        text = stringResource(id = R.string.number_of_providers, providers),
        onRemoveClick = onRemoveClick,
        enabled = true,
    )
}

@Composable
fun OwnershipFilterChip(ownership: Ownership, onRemoveClick: () -> Unit) {
    MullvadFilterChip(
        text = stringResource(ownership.stringResources()),
        onRemoveClick = onRemoveClick,
        enabled = true,
    )
}

@Composable
fun DaitaFilterChip() {
    MullvadFilterChip(
        text = stringResource(id = R.string.setting_chip, stringResource(id = R.string.daita)),
        onRemoveClick = {},
        enabled = false,
    )
}

@Composable
fun EntryFilterChip() {
    MullvadFilterChip(
        text = stringResource(id = R.string.entry),
        onRemoveClick = {},
        enabled = false,
    )
}

@Composable
fun ExitFilterChip() {
    MullvadFilterChip(
        text = stringResource(id = R.string.exit),
        onRemoveClick = {},
        enabled = false,
    )
}

private fun Ownership.stringResources(): Int =
    when (this) {
        Ownership.MullvadOwned -> R.string.owned
        Ownership.Rented -> R.string.rented
    }
