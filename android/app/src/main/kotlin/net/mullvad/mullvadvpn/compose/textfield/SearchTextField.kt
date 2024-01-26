package net.mullvad.mullvadvpn.compose.textfield

import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.text.BasicTextField
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextFieldDefaults
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.ColorFilter
import androidx.compose.ui.graphics.SolidColor
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.input.VisualTransformation
import androidx.compose.ui.tooling.preview.Preview
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens

@Preview
@Composable
private fun PreviewSearchTextField() {
    AppTheme {
        Column(modifier = Modifier.background(color = MaterialTheme.colorScheme.background)) {
            SearchTextField(
                placeHolder = "Search for...",
                backgroundColor = MaterialTheme.colorScheme.tertiaryContainer,
                textColor = MaterialTheme.colorScheme.onTertiaryContainer,
            ) {}
        }
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun SearchTextField(
    modifier: Modifier = Modifier,
    placeHolder: String = stringResource(id = R.string.search_placeholder),
    backgroundColor: Color,
    textColor: Color,
    enabled: Boolean = true,
    singleLine: Boolean = true,
    interactionSource: MutableInteractionSource = remember { MutableInteractionSource() },
    visualTransformation: VisualTransformation = VisualTransformation.None,
    onValueChange: (String) -> Unit
) {
    var searchTerm by rememberSaveable { mutableStateOf("") }

    BasicTextField(
        value = searchTerm,
        textStyle = MaterialTheme.typography.labelLarge.copy(color = textColor),
        onValueChange = { text: String ->
            searchTerm = text
            onValueChange.invoke(text)
        },
        singleLine = singleLine,
        cursorBrush = SolidColor(textColor),
        decorationBox =
            @Composable { innerTextField ->
                TextFieldDefaults.DecorationBox(
                    value = searchTerm,
                    innerTextField = innerTextField,
                    enabled = enabled,
                    singleLine = singleLine,
                    interactionSource = interactionSource,
                    visualTransformation = visualTransformation,
                    leadingIcon = {
                        Image(
                            painter = painterResource(id = R.drawable.icons_search),
                            contentDescription = null,
                            modifier =
                                Modifier.size(
                                    width = Dimens.searchIconSize,
                                    height = Dimens.searchIconSize,
                                ),
                            colorFilter = ColorFilter.tint(color = textColor),
                        )
                    },
                    placeholder = {
                        Text(text = placeHolder, style = MaterialTheme.typography.labelLarge)
                    },
                    trailingIcon = {
                        if (searchTerm.isNotEmpty()) {
                            Image(
                                modifier =
                                    Modifier.size(Dimens.smallIconSize).clickable {
                                        searchTerm = ""
                                        onValueChange.invoke(searchTerm)
                                    },
                                painter = painterResource(id = R.drawable.icon_close),
                                contentDescription = null,
                            )
                        }
                    },
                    shape = MaterialTheme.shapes.medium,
                    colors =
                        TextFieldDefaults.colors(
                            focusedTextColor = textColor,
                            unfocusedTextColor = textColor,
                            focusedContainerColor = backgroundColor,
                            unfocusedContainerColor = backgroundColor,
                            focusedIndicatorColor = Color.Transparent,
                            unfocusedIndicatorColor = Color.Transparent,
                            cursorColor = textColor,
                            focusedPlaceholderColor = textColor,
                            unfocusedPlaceholderColor = textColor
                        ),
                    contentPadding = PaddingValues(),
                )
            },
        modifier = modifier
    )
}
