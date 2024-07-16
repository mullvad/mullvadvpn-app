package net.mullvad.mullvadvpn.compose.bottomsheet

import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.stringResource
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.annotation.RootGraph
import com.ramcosta.composedestinations.bottomsheet.spec.DestinationStyleBottomSheet
import com.ramcosta.composedestinations.generated.destinations.CreateCustomListDestination
import com.ramcosta.composedestinations.generated.destinations.CustomListsSheetDestination
import com.ramcosta.composedestinations.generated.destinations.CustomListsDestination
import com.ramcosta.composedestinations.navigation.DestinationsNavigator
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.cell.HeaderCell
import net.mullvad.mullvadvpn.compose.cell.IconCell
import net.mullvad.mullvadvpn.compose.component.MullvadModalBottomSheet
import net.mullvad.mullvadvpn.lib.theme.color.AlphaInactive
import net.mullvad.mullvadvpn.lib.theme.color.AlphaVisible

@OptIn(ExperimentalMaterial3Api::class)
@Destination<RootGraph>(style = DestinationStyleBottomSheet::class)
@Composable
fun CustomListsSheet(navigator: DestinationsNavigator, editListEnabled: Boolean) {
    MullvadModalBottomSheet {
        HeaderCell(
            text = stringResource(id = R.string.edit_custom_lists),
            background = Color.Unspecified
        )
        HorizontalDivider(color = MaterialTheme.colorScheme.onBackground)
        IconCell(
            iconId = R.drawable.icon_add,
            title = stringResource(id = R.string.new_list),
            titleColor = MaterialTheme.colorScheme.onBackground,
            onClick = {
                navigator.navigate(CreateCustomListDestination(null)) {
                    popUpTo(CustomListsSheetDestination) { inclusive = true }
                }
            },
            background = Color.Unspecified
        )
        IconCell(
            iconId = R.drawable.icon_edit,
            title = stringResource(id = R.string.edit_lists),
            titleColor =
                MaterialTheme.colorScheme.onBackground.copy(
                    alpha =
                        if (editListEnabled) {
                            AlphaVisible
                        } else {
                            AlphaInactive
                        }
                ),
            onClick = { navigator.navigate(CustomListsDestination) },
            background = Color.Unspecified,
            enabled = editListEnabled
        )
    }
}
