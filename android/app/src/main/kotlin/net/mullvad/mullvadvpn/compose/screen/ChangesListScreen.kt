package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.colorResource
import androidx.constraintlayout.compose.ConstraintLayout
import net.mullvad.mullvadvpn.BuildConfig
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.component.ShowChangesDialog
import net.mullvad.mullvadvpn.repository.ChangeLogState
import net.mullvad.mullvadvpn.viewmodel.AppChangesViewModel

@Composable
fun ChangesListScreen(
    viewModel: AppChangesViewModel,
    onBackPressed: () -> Unit
) {
    val state = viewModel.changeLogState.collectAsState().value
    if (state == ChangeLogState.ShouldShow) {

        ShowChangesDialog(
            changesList = viewModel.getChangesList(),
            version = BuildConfig.VERSION_NAME,
            onDismiss = {
                onBackPressed()
            }
        )
    }

    ConstraintLayout(
        modifier = Modifier
            .fillMaxHeight()
            .fillMaxWidth()
            .background(colorResource(id = R.color.colorPrimary))
    ) {}
}
