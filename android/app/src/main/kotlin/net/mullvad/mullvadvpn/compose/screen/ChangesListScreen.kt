package net.mullvad.mullvadvpn.compose.screen

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.colorResource
import androidx.compose.ui.unit.dp
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
    ) {
        val (title, changes, back) = createRefs()

        Column(
            Modifier
                .padding(start = 16.dp, end = 16.dp, bottom = 16.dp)
                .height(44.dp)
                .constrainAs(back) {
                    start.linkTo(parent.start, margin = 16.dp)
                    end.linkTo(parent.end, margin = 16.dp)
                    bottom.linkTo(parent.bottom, margin = 12.dp)
                }
        ) {
        }
    }
}
