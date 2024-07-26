package net.mullvad.mullvadvpn.compose.component

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.asPaddingValues
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.material3.BottomSheetDefaults
import androidx.compose.material3.BottomSheetDefaults.DragHandle
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment.Companion.CenterHorizontally
import androidx.compose.ui.Modifier
import androidx.compose.ui.tooling.preview.Preview
import net.mullvad.mullvadvpn.compose.cell.HeaderCell
import net.mullvad.mullvadvpn.compose.cell.IconCell
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens

@OptIn(ExperimentalMaterial3Api::class)
@Preview
@Composable
private fun PreviewMullvadModalBottomSheet() {
    AppTheme {
        MullvadModalBottomContainer {
            HeaderCell(
                text = "Title",
            )
            HorizontalDivider()
            IconCell(
                iconId = null,
                title = "Select",
            )
        }
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun MullvadModalBottomContainer(sheetContent: @Composable ColumnScope.() -> Unit) {
    val paddingValues = BottomSheetDefaults.windowInsets.asPaddingValues()
    Column(Modifier.fillMaxWidth()) {
        DragHandle(
            modifier = Modifier.align(CenterHorizontally),
            color = MaterialTheme.colorScheme.onSurface
        )
        sheetContent()
        Spacer(modifier = Modifier.height(Dimens.smallPadding))
        Spacer(modifier = Modifier.height(paddingValues.calculateBottomPadding()))
    }
}
