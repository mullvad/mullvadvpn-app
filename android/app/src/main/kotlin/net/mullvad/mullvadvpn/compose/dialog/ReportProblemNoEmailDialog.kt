package net.mullvad.mullvadvpn.compose.dialog

import androidx.compose.foundation.Image
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.width
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.colorResource
import androidx.compose.ui.res.dimensionResource
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.button.ActionButton

@Preview
@Composable
private fun PreviewReportProblemNoEmailDialog() {
    ReportProblemNoEmailDialog(
        onDismiss = {},
        onConfirm = {},
    )
}

@Composable
fun ReportProblemNoEmailDialog(onDismiss: () -> Unit, onConfirm: () -> Unit) {
    AlertDialog(
        onDismissRequest = { onDismiss() },
        title = {
            Column(
                horizontalAlignment = Alignment.CenterHorizontally,
                modifier = Modifier.fillMaxWidth()
            ) {
                Image(
                    painter = painterResource(id = R.drawable.icon_alert),
                    contentDescription = "Remove",
                    modifier = Modifier.width(50.dp).height(50.dp)
                )
            }
        },
        text = {
            Text(
                text = stringResource(id = R.string.confirm_no_email),
                color = colorResource(id = R.color.white),
                fontSize = dimensionResource(id = R.dimen.text_small).value.sp,
                modifier = Modifier.fillMaxWidth()
            )
        },
        dismissButton = {
            ActionButton(
                modifier = Modifier.fillMaxWidth(),
                colors =
                    ButtonDefaults.buttonColors(
                        containerColor = colorResource(id = R.color.red),
                        contentColor = Color.White
                    ),
                onClick = onConfirm,
                text = stringResource(id = R.string.send_anyway)
            )
        },
        confirmButton = {
            ActionButton(
                modifier = Modifier.fillMaxWidth(),
                colors =
                    ButtonDefaults.buttonColors(
                        containerColor = colorResource(id = R.color.blue),
                        contentColor = Color.White
                    ),
                onClick = { onDismiss() },
                text = stringResource(id = R.string.back)
            )
        },
        containerColor = colorResource(id = R.color.darkBlue)
    )
}
