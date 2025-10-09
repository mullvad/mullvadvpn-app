package net.mullvad.mullvadvpn.compose.util

import android.content.Context
import android.content.Intent
import android.net.Uri
import androidx.activity.result.contract.ActivityResultContract

open class SendEmail : ActivityResultContract<EmailData, Void?>() {

    override fun createIntent(context: Context, input: EmailData): Intent {
        val intent = Intent(Intent.ACTION_SEND)
        intent.setType(EMAIL_TYPE)
        intent.putExtra(Intent.EXTRA_EMAIL, input.to.toTypedArray())
        intent.putExtra(Intent.EXTRA_SUBJECT, input.subject)
        if (input.body != null) {
            intent.putExtra(Intent.EXTRA_TEXT, input.body)
        }
        if (input.attachment != null) {
            intent.putExtra(Intent.EXTRA_STREAM, input.attachment)
            intent.addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION)
        }
        return intent
    }

    final override fun getSynchronousResult(
        context: Context,
        input: EmailData,
    ): SynchronousResult<Void?>? = null

    final override fun parseResult(resultCode: Int, intent: Intent?): Void? {
        return null
    }

    companion object {
        const val EMAIL_TYPE = "text/plain"
    }
}

data class EmailData(
    val to: List<String>,
    val subject: String,
    val body: String? = null,
    val attachment: Uri? = null,
)
