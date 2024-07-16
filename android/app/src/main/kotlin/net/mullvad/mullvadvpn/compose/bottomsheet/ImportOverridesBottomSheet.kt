package net.mullvad.mullvadvpn.compose.bottomsheet

import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextOverflow
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.compose.dropUnlessResumed
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.annotation.RootGraph
import com.ramcosta.composedestinations.bottomsheet.spec.DestinationStyleBottomSheet
import com.ramcosta.composedestinations.generated.destinations.ImportOverridesByTextDestination
import com.ramcosta.composedestinations.navigation.DestinationsNavigator
import com.ramcosta.composedestinations.result.ResultBackNavigator
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.cell.HeaderCell
import net.mullvad.mullvadvpn.compose.cell.IconCell
import net.mullvad.mullvadvpn.compose.component.MullvadModalBottomSheet
import net.mullvad.mullvadvpn.compose.test.SERVER_IP_OVERRIDES_IMPORT_BY_FILE_TEST_TAG
import net.mullvad.mullvadvpn.compose.test.SERVER_IP_OVERRIDES_IMPORT_BY_TEXT_TEST_TAG
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.viewmodel.ImportOverridesSheetViewModel
import org.koin.androidx.compose.koinViewModel

@OptIn(ExperimentalMaterial3Api::class)
@Destination<RootGraph>(style = DestinationStyleBottomSheet::class)
@Composable
fun ImportOverridesSheet(
    navigator: DestinationsNavigator,
    resultRecipient: ResultBackNavigator<Boolean>
) {
    val vm = koinViewModel<ImportOverridesSheetViewModel>()
    val state = vm.uiState.collectAsStateWithLifecycle()

    MullvadModalBottomSheet {
        HeaderCell(
            text = stringResource(id = R.string.server_ip_overrides_import_by),
            background = Color.Unspecified
        )
        HorizontalDivider(color = MaterialTheme.colorScheme.onBackground)
        IconCell(
            iconId = R.drawable.icon_upload_file,
            title = stringResource(id = R.string.server_ip_overrides_import_by_file),
            onClick = dropUnlessResumed { resultRecipient.navigateBack(true) },
            background = Color.Unspecified,
            modifier = Modifier.testTag(SERVER_IP_OVERRIDES_IMPORT_BY_FILE_TEST_TAG)
        )
        IconCell(
            iconId = R.drawable.icon_text_fields,
            title = stringResource(id = R.string.server_ip_overrides_import_by_text),
            onClick = dropUnlessResumed { navigator.navigate(ImportOverridesByTextDestination) },
            background = Color.Unspecified,
            modifier = Modifier.testTag(SERVER_IP_OVERRIDES_IMPORT_BY_TEXT_TEST_TAG)
        )
        if (state.value.overridesActive) {
            HorizontalDivider(color = MaterialTheme.colorScheme.onBackground)
            Row(
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Icon(
                    modifier = Modifier.padding(Dimens.mediumPadding),
                    painter = painterResource(id = R.drawable.icon_info),
                    tint = MaterialTheme.colorScheme.errorContainer,
                    contentDescription = null
                )
                Text(
                    modifier =
                        Modifier.padding(
                            top = Dimens.smallPadding,
                            end = Dimens.mediumPadding,
                            bottom = Dimens.smallPadding
                        ),
                    text = stringResource(R.string.import_overrides_bottom_sheet_override_warning),
                    maxLines = 2,
                    style = MaterialTheme.typography.bodySmall,
                    overflow = TextOverflow.Ellipsis,
                )
            }
        }
    }
}
