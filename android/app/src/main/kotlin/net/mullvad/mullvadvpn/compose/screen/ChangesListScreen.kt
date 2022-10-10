package net.mullvad.mullvadvpn.compose.screen

import android.content.Context
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.colorResource
import androidx.compose.ui.unit.dp
import androidx.constraintlayout.compose.ConstraintLayout
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.component.ShowChangesDialog
import net.mullvad.mullvadvpn.ui.serviceconnection.ServiceConnectionManager
import net.mullvad.mullvadvpn.viewmodel.AppChangesViewModel

@Composable
fun ChangesListScreen(
    context: Context,
    viewModel: AppChangesViewModel,
    serviceConnectionManager: ServiceConnectionManager,
    onBackPressed: () -> Unit
) {

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
            ShowChangesDialog(
                context = context,
                changesViewModel = viewModel,
                serviceConnectionManager = serviceConnectionManager,
                onBackPressed = onBackPressed
            )
        }
    }
}
