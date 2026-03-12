package net.mullvad.mullvadvpn.feature.login.impl

import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.annotation.ExternalModuleGraph
import com.ramcosta.composedestinations.spec.DestinationStyle
import kotlinx.serialization.Serializable
import net.mullvad.mullvadvpn.core.nav3.LocalResultStore
import net.mullvad.mullvadvpn.core.nav3.NavResult
import net.mullvad.mullvadvpn.core.nav3.Navigator
import net.mullvad.mullvadvpn.lib.ui.component.dialog.InfoConfirmationDialog
import net.mullvad.mullvadvpn.lib.ui.component.dialog.InfoConfirmationDialogTitleType
import net.mullvad.mullvadvpn.lib.ui.theme.Dimens

@Preview
@Composable
private fun PreviewCreateAccountConfirmationDialog() {
    //        AppTheme { CreateAccountConfirmation(EmptyResultBackNavigator()) }
}

@Serializable
data class CreateAccountConfirmationDialogResult(val confirmed: Boolean) : NavResult

@Composable
@Destination<ExternalModuleGraph>(style = DestinationStyle.Dialog::class)
fun CreateAccountConfirmation(navigator: Navigator) {
    val resultStore = LocalResultStore.current
    InfoConfirmationDialog(
        onResult = {
            if (it != null) {
                navigator.goBack(resultStore, result = CreateAccountConfirmationDialogResult(true))
            } else {
                navigator.goBack(resultStore, result = CreateAccountConfirmationDialogResult(false))
            }
        },
        titleType = InfoConfirmationDialogTitleType.IconOnly,
        confirmButtonTitle = stringResource(R.string.create_new_account),
        cancelButtonTitle = stringResource(R.string.cancel),
    ) {
        Text(
            text = stringResource(id = R.string.create_new_account_warning_paragraph1),
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            style = MaterialTheme.typography.labelLarge,
            modifier = Modifier.fillMaxWidth(),
        )

        Spacer(modifier = Modifier.height(Dimens.verticalSpace))

        Text(
            text = stringResource(id = R.string.create_new_account_warning_paragraph2),
            color = MaterialTheme.colorScheme.onSurfaceVariant,
            style = MaterialTheme.typography.labelLarge,
            modifier = Modifier.fillMaxWidth(),
        )
    }
}
