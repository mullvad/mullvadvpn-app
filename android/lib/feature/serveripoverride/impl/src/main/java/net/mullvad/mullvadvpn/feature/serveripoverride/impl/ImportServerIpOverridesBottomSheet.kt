package net.mullvad.mullvadvpn.feature.serveripoverride.impl

import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.padding
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Info
import androidx.compose.material.icons.rounded.TextFields
import androidx.compose.material.icons.rounded.UploadFile
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.rememberModalBottomSheetState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextOverflow
import androidx.lifecycle.compose.dropUnlessResumed
import net.mullvad.mullvadvpn.common.compose.navigateReplaceTop
import net.mullvad.mullvadvpn.core.Navigator
import net.mullvad.mullvadvpn.feature.serveripoverride.api.ImportOverrideByFileNavResult
import net.mullvad.mullvadvpn.feature.serveripoverride.api.ImportOverrideByTextNavKey
import net.mullvad.mullvadvpn.lib.ui.component.MullvadModalBottomSheet
import net.mullvad.mullvadvpn.lib.ui.component.listitem.BottomSheetListItem
import net.mullvad.mullvadvpn.lib.ui.component.listitem.IconListItem
import net.mullvad.mullvadvpn.lib.ui.designsystem.ListItemDefaults
import net.mullvad.mullvadvpn.lib.ui.designsystem.Position
import net.mullvad.mullvadvpn.lib.ui.resource.R
import net.mullvad.mullvadvpn.lib.ui.tag.SERVER_IP_OVERRIDES_IMPORT_BY_FILE_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.tag.SERVER_IP_OVERRIDES_IMPORT_BY_TEXT_TEST_TAG
import net.mullvad.mullvadvpn.lib.ui.theme.Dimens

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ImportOverridesBottomSheet(navigator: Navigator, overridesActive: Boolean) {
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
            overridesActive = overridesActive,
            onImportByFile =
                dropUnlessResumed { navigator.goBack(result = ImportOverrideByFileNavResult) },
            onImportByText =
                dropUnlessResumed {
                    navigator.navigateReplaceTop(
                        sheetState, scope,
                        ImportOverrideByTextNavKey
                    )
                },
        )
    }
}

@Composable
private fun BottomSheetContent(
    backgroundColor: Color,
    onBackgroundColor: Color,
    overridesActive: Boolean,
    onImportByFile: () -> Unit,
    onImportByText: () -> Unit,
) {
    BottomSheetListItem(
        title = stringResource(id = R.string.server_ip_overrides_import_by),
        backgroundColor = backgroundColor,
        onBackgroundColor = onBackgroundColor,
    )
    HorizontalDivider(color = onBackgroundColor)
    IconListItem(
        leadingIcon = Icons.Rounded.UploadFile,
        title = stringResource(id = R.string.server_ip_overrides_import_by_file),
        modifier = Modifier.testTag(SERVER_IP_OVERRIDES_IMPORT_BY_FILE_TEST_TAG),
        position = Position.Middle,
        onClick = { onImportByFile() },
        colors =
            ListItemDefaults.colors(
                containerColorParent = backgroundColor,
                headlineColor = onBackgroundColor,
            ),
    )
    IconListItem(
        leadingIcon = Icons.Rounded.TextFields,
        title = stringResource(id = R.string.server_ip_overrides_import_by_text),
        position = Position.Middle,
        modifier = Modifier.testTag(SERVER_IP_OVERRIDES_IMPORT_BY_TEXT_TEST_TAG),
        onClick = { onImportByText() },
        colors =
            ListItemDefaults.colors(
                containerColorParent = backgroundColor,
                headlineColor = onBackgroundColor,
            ),
    )
    if (overridesActive) {
        HorizontalDivider(color = onBackgroundColor)
        Row(verticalAlignment = Alignment.CenterVertically) {
            Icon(
                modifier = Modifier.padding(Dimens.mediumPadding),
                imageVector = Icons.Rounded.Info,
                tint = MaterialTheme.colorScheme.error,
                contentDescription = null,
            )
            Text(
                modifier =
                    Modifier.padding(
                        top = Dimens.smallPadding,
                        end = Dimens.mediumPadding,
                        bottom = Dimens.smallPadding,
                    ),
                text = stringResource(R.string.import_overrides_bottom_sheet_override_warning),
                maxLines = 2,
                style = MaterialTheme.typography.labelLarge,
                overflow = TextOverflow.Ellipsis,
            )
        }
    }
}
