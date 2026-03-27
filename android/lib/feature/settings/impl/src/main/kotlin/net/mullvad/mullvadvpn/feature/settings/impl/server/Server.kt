package net.mullvad.mullvadvpn.feature.settings.impl.server

import android.content.Context
import androidx.compose.material3.MaterialTheme
import androidx.compose.remote.creation.compose.capture.CapturedDocument
import androidx.compose.remote.creation.compose.capture.captureSingleRemoteDocument
import androidx.compose.remote.creation.compose.layout.RemoteArrangement
import androidx.compose.remote.creation.compose.layout.RemoteColumn
import androidx.compose.remote.creation.compose.layout.RemoteText
import androidx.compose.remote.creation.compose.state.RemoteDp
import androidx.compose.remote.creation.compose.state.asRemoteTextUnit
import androidx.compose.remote.creation.compose.state.rc
import androidx.compose.remote.creation.compose.text.RemoteTextStyle
import androidx.compose.runtime.Composable
import androidx.compose.ui.unit.dp
import co.touchlab.kermit.Logger
import net.mullvad.mullvadvpn.feature.settings.impl.server.model.FaqItem

class Server(private val context: Context) {
    suspend fun main(): CapturedDocument {
        val document = download()
        val data = parseFaqClean(document)
        val captured = captureSingleRemoteDocument(context = context) { RemoteFaq(faqItems = data) }
        return captured
    }
}

@Composable
fun RemoteFaq(faqItems: List<FaqItem>) {
    RemoteColumn(verticalArrangement = RemoteArrangement.spacedBy(RemoteDp(10.dp))) {
        RemoteText(
            "These are FAQs",
            color = MaterialTheme.colorScheme.onPrimary.rc,
            fontSize = MaterialTheme.typography.titleLarge.fontSize.asRemoteTextUnit(),
        )
        Logger.d { "questions=$faqItems" }
        faqItems.forEach { q ->
            RemoteText(
                text = "Section",
                style = RemoteTextStyle.Default,
                color = MaterialTheme.colorScheme.onPrimary.rc,
            )

            RemoteText(
                text = q.question,
                style = RemoteTextStyle.Default,
                color = MaterialTheme.colorScheme.onPrimary.rc,
            )

            RemoteText(
                text = q.answer,
                style = RemoteTextStyle.Default,
                color = MaterialTheme.colorScheme.onPrimary.rc,
            )

            /*q.content.forEach { c ->
                when (c) {
                    is FaqBlock.Content.Paragraph ->
                        RemoteText(c.text, color = MaterialTheme.colorScheme.onPrimary.rc)

                    is FaqBlock.Content.ListItem ->
                        RemoteText("• ${c.text}", color = MaterialTheme.colorScheme.onPrimary.rc)
                }
            }*/
        }
    }
}
