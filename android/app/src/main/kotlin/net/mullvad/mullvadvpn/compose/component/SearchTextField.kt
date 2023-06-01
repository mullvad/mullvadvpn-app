package net.mullvad.mullvadvpn.compose.component

import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.TextField
import androidx.compose.runtime.Composable

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun SearchTextField(text: String = "", placeHolder: String = "", onValueChange: (String) -> Unit) {
    TextField(value = text, onValueChange = onValueChange)
}
