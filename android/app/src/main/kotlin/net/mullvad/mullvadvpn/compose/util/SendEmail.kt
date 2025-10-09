package net.mullvad.mullvadvpn.compose.util

import android.content.Context
import android.content.Intent
import android.net.Uri
import androidx.activity.result.contract.ActivityResultContract
import androidx.core.net.toUri
import kotlin.collections.toTypedArray

open class SendEmail : ActivityResultContract<EmailData, Void?>() {

    override fun createIntent(context: Context, input: EmailData): Intent {
        val intent =
            if (input.attachment != null) {
                emailIntentWithAttachment().let {
                    it.putExtra(Intent.EXTRA_STREAM, input.attachment)
                    it.addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION)
                }
            } else {
                emailIntentWithoutAttachment()
            }
        intent.putExtra(Intent.EXTRA_EMAIL, input.to.toTypedArray())
        if (input.subject != null) {
            intent.putExtra(Intent.EXTRA_SUBJECT, input.subject)
        }
        if (input.body != null) {
            intent.putExtra(Intent.EXTRA_TEXT, input.body)
        }
        return intent
    }

    /**
     * Create email intent with attachment Due to a limitation with attachments we can only use
     * ACTION_SEND i.e. a generic share intent This causes the list of apps to choose from to be
     * larger than just email apps
     */
    private fun emailIntentWithAttachment(): Intent {
        val intent = Intent(Intent.ACTION_SEND)
        intent.setType(EMAIL_TYPE)
        return intent
    }

    /**
     * Create email intent without attachment This uses the standard email intent ACTION_SENDTO with
     * a mailto: Uri This limits the list of apps to choose from to email apps only
     */
    private fun emailIntentWithoutAttachment(): Intent {
        val intent = Intent(Intent.ACTION_SENDTO)
        intent.setData("mailto:".toUri())
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
    val subject: String? = null,
    val body: String? = null,
    val attachment: Uri? = null,
)
