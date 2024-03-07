package net.mullvad.mullvadvpn.compose.component

import androidx.compose.foundation.layout.ColumnScope
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.asPaddingValues
import androidx.compose.foundation.layout.height
import androidx.compose.material3.BottomSheetDefaults
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.ModalBottomSheet
import androidx.compose.material3.SheetState
import androidx.compose.material3.rememberModalBottomSheetState
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
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
        MullvadModalBottomSheet(
            sheetContent = {
                HeaderCell(
                    text = "Title",
                )
                HorizontalDivider()
                IconCell(
                    iconId = null,
                    title = "Select",
                )
            },
            closeBottomSheet = {}
        )
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun MullvadModalBottomSheet(
    modifier: Modifier = Modifier,
    sheetState: SheetState = rememberModalBottomSheetState(skipPartiallyExpanded = true),
    backgroundColor: Color = MaterialTheme.colorScheme.surfaceContainer,
    onBackgroundColor: Color = MaterialTheme.colorScheme.onSurface,
    closeBottomSheet: () -> Unit,
    sheetContent: @Composable ColumnScope.() -> Unit
) {
    // This is to avoid weird colors in the status bar and the navigation bar
    val paddingValues = BottomSheetDefaults.windowInsets.asPaddingValues()
    ModalBottomSheet(
        onDismissRequest = closeBottomSheet,
        sheetState = sheetState,
        containerColor = backgroundColor,
        modifier = modifier,
        windowInsets = WindowInsets(0, 0, 0, 0), // No insets
        dragHandle = { BottomSheetDefaults.DragHandle(color = onBackgroundColor) }
    ) {
        sheetContent()
        Spacer(modifier = Modifier.height(Dimens.smallPadding))
        Spacer(modifier = Modifier.height(paddingValues.calculateBottomPadding()))
    }
}
