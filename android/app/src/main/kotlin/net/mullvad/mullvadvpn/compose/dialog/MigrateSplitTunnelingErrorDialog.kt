package net.mullvad.mullvadvpn.compose.dialog

import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.compositeOver
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.tooling.preview.Preview
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.navigation.DestinationsNavigator
import com.ramcosta.composedestinations.spec.DestinationStyle
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.button.PrimaryButton
import net.mullvad.mullvadvpn.compose.util.LaunchedEffectCollect
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.color.AlphaDescription
import net.mullvad.mullvadvpn.viewmodel.MigrateSplitTunnelingErrorUiSideEffect
import net.mullvad.mullvadvpn.viewmodel.MigrateSplitTunnelingErrorViewModel
import org.koin.androidx.compose.koinViewModel

@Preview
@Composable
private fun PreviewMigrateSplitTunnelingErrorDialog() {
    AppTheme {
        MigrateSplitTunnelingErrorDialog(
            onCloseDialog = {},
            onTryAgainLater = {},
            onNeverTryAgain = {},
        )
    }
}

@Destination(style = DestinationStyle.Dialog::class)
@Composable
fun MigrateSplitTunnelingError(navigator: DestinationsNavigator) {
    val vm = koinViewModel<MigrateSplitTunnelingErrorViewModel>()

    LaunchedEffectCollect(sideEffect = vm.uiSideEffect) {
        when (it) {
            MigrateSplitTunnelingErrorUiSideEffect.CloseScreen -> navigator.navigateUp()
        }
    }

    MigrateSplitTunnelingErrorDialog(
        onCloseDialog = navigator::navigateUp,
        onTryAgainLater = vm::tryAgainLater,
        onNeverTryAgain = vm::clearOldSettings,
    )
}

@Composable
fun MigrateSplitTunnelingErrorDialog(
    onCloseDialog: () -> Unit,
    onTryAgainLater: () -> Unit,
    onNeverTryAgain: () -> Unit
) {
    AlertDialog(
        icon = {
            Icon(
                painter = painterResource(id = R.drawable.icon_fail),
                contentDescription = null,
            )
        },
        title = {
            // TODO translate
            Text(
                text = "Migrate split tunneling error",
                style = MaterialTheme.typography.headlineSmall,
            )
        },
        text = {
            // TODO translate
            Text(
                text = "There was an error while trying to migrate split tunneling settings",
                style = MaterialTheme.typography.bodySmall,
            )
        },
        containerColor = MaterialTheme.colorScheme.background,
        titleContentColor = MaterialTheme.colorScheme.onBackground,
        iconContentColor = Color.Unspecified,
        textContentColor =
            MaterialTheme.colorScheme.onBackground
                .copy(alpha = AlphaDescription)
                .compositeOver(MaterialTheme.colorScheme.background),
        onDismissRequest = onCloseDialog,
        dismissButton = {
            PrimaryButton(
                text = "Never try again",
                onClick = onNeverTryAgain,
            )
        },
        confirmButton = {
            PrimaryButton(
                // TODO Translate
                text = "Try again later",
                onClick = onTryAgainLater,
            )
        },
    )
}
