package net.mullvad.mullvadvpn.lib.feature.customvpn.impl.bottomsheet

import androidx.compose.foundation.layout.padding
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Add
import androidx.compose.material.icons.outlined.Folder
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.rememberModalBottomSheetState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.tooling.preview.Preview
import androidx.lifecycle.compose.dropUnlessResumed
import net.mullvad.mullvadvpn.core.EmptyNavigator
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.feature.customvpn.api.EditCustomTunnelNavKey
import net.mullvad.mullvadvpn.feature.customvpn.impl.R
import net.mullvad.mullvadvpn.lib.common.compose.navigateReplaceTop
import net.mullvad.mullvadvpn.lib.ui.component.MullvadModalBottomSheet
import net.mullvad.mullvadvpn.lib.ui.component.listitem.IconListItem
import net.mullvad.mullvadvpn.lib.ui.designsystem.ListItemDefaults
import net.mullvad.mullvadvpn.lib.ui.designsystem.Position
import net.mullvad.mullvadvpn.lib.ui.theme.AppTheme
import net.mullvad.mullvadvpn.lib.ui.theme.Dimens

@Preview
@Composable
private fun PreviewAddCustomTunnelBottomSheet() {
    AppTheme {
        AddCustomTunnelBottomSheet(navigator = EmptyNavigator)
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun AddCustomTunnelBottomSheet(navigator: Navigator) {
    val sheetState = rememberModalBottomSheetState(skipPartiallyExpanded = true)
    val scope = rememberCoroutineScope()

    val backgroundColor: Color = MaterialTheme.colorScheme.surfaceContainer
    val onBackgroundColor: Color = MaterialTheme.colorScheme.onSurface

    MullvadModalBottomSheet(
        sheetState = sheetState,
        backgroundColor = backgroundColor,
        onBackgroundColor = onBackgroundColor,
        onDismissRequest = { navigator.goBack() },
    ) {
        BottomSheetContent(
            backgroundColor = backgroundColor,
            onBackgroundColor = onBackgroundColor,
            onCreateConfig =
                dropUnlessResumed {
                    navigator.navigateReplaceTop(
                        sheetState,
                        scope,
                        EditCustomTunnelNavKey(new = true),
                    )
                },
            onImportByFile = dropUnlessResumed { navigator.goBack() },
        )
    }
}

@Composable
private fun BottomSheetContent(
    backgroundColor: Color,
    onBackgroundColor: Color,
    onCreateConfig: () -> Unit,
    onImportByFile: () -> Unit,
) {
    Header(
        text = stringResource(id = R.string.add_new_configuration),
        color = onBackgroundColor,
    )
    Experimental(
        text = stringResource(id = R.string.custom_tunnels_experimental_warning),
        color = onBackgroundColor,
    )
    HorizontalDivider(
        color = onBackgroundColor,
        modifier = Modifier.padding(horizontal = Dimens.sideMarginNew),
    )
    IconListItem(
        leadingIcon = Icons.Default.Add,
        title = stringResource(id = R.string.create),
        position = Position.Middle,
        onClick = { onCreateConfig() },
        colors =
            ListItemDefaults.colors(
                containerColorParent = backgroundColor,
                headlineColor = onBackgroundColor,
            ),
    )
    IconListItem(
        leadingIcon = Icons.Outlined.Folder,
        title = stringResource(id = R.string.import_wireguard_configuration),
        position = Position.Middle,
        onClick = { onImportByFile() },
        colors =
            ListItemDefaults.colors(
                containerColorParent = backgroundColor,
                headlineColor = onBackgroundColor,
            ),
    )
}

@Composable
private fun Experimental(text: String, color: Color) {
    Text(
        text = text,
        color = color,
        style = MaterialTheme.typography.labelSmall.merge(fontWeight = FontWeight.Normal),
        modifier =
            Modifier.padding(horizontal = Dimens.sideMarginNew, vertical = Dimens.smallPadding),
    )
}

@Composable
private fun Header(text: String, color: Color) {
    Text(
        text = text,
        color = color,
        style = MaterialTheme.typography.headlineSmall,
        modifier =
            Modifier.padding(horizontal = Dimens.sideMarginNew, vertical = Dimens.smallPadding),
    )
}
