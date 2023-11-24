package net.mullvad.mullvadvpn.compose.cell

import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
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
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.model.Ownership

@Preview
@Composable
private fun PreviewFilterCell() {
    AppTheme {
        FilterCell(
            ownershipFilter = Ownership.MullvadOwned,
            selectedProviderFilter = 3,
            removeOwnershipFilter = {},
            removeProviderFilter = {}
        )
    }
}

@Composable
fun FilterCell(
    ownershipFilter: Ownership?,
    selectedProviderFilter: Int?,
    removeOwnershipFilter: () -> Unit,
    removeProviderFilter: () -> Unit
) {
    val scrollState = rememberScrollState()
    Row(
        verticalAlignment = Alignment.CenterVertically,
        modifier =
            Modifier.horizontalScroll(scrollState)
                .padding(
                    horizontal = Dimens.searchFieldHorizontalPadding,
                    vertical = Dimens.selectLocationTitlePadding
                )
                .fillMaxWidth(),
    ) {
        Text(
            modifier = Modifier.padding(end = Dimens.textEndPadding),
            text = stringResource(id = R.string.filtered),
            color = MaterialTheme.colorScheme.onPrimary,
            style = MaterialTheme.typography.labelMedium
        )

        if (selectedProviderFilter != null) {
            MullvadFilterChip(
                text = stringResource(id = R.string.number_of_providers, selectedProviderFilter),
                onRemoveClick = removeProviderFilter
            )
            Spacer(modifier = Modifier.size(Dimens.chipSpace))
        }

        if (ownershipFilter != null) {
            MullvadFilterChip(
                text = stringResource(ownershipFilter.stringResources()),
                onRemoveClick = removeOwnershipFilter
            )
        }
    }
}

private fun Ownership.stringResources(): Int =
    when (this) {
        Ownership.MullvadOwned -> R.string.owned
        Ownership.Rented -> R.string.rented
    }
