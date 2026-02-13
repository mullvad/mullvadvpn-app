package net.mullvad.mullvadvpn.feature.login.impl.apiunreachable

import android.content.Context
import android.content.Intent
import android.net.Uri
import androidx.activity.result.contract.ActivityResultContract
import androidx.core.app.ShareCompat
import androidx.core.net.toUri
import kotlin.collections.toTypedArray

class SendEmail : ActivityResultContract<EmailData, Unit>() {

    /**
     * Create email intent with or without attachment Depending on whether an attachment is provided
     * we either create a generic share intent (with attachment) or a standard email intent (without
     * attachment)
     */
    override fun createIntent(context: Context, input: EmailData): Intent {
        return ShareCompat.IntentBuilder(context)
            .setEmailTo(input.to.toTypedArray())
            .setSubject(input.subject)
            .setText(input.body)
            .setType(EMAIL_TYPE)
            .intent
            .let {
                if (input.attachment != null) {
                    it.putExtra(Intent.EXTRA_STREAM, input.attachment)
                        .addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION)
                } else {
                    it.setAction(Intent.ACTION_SENDTO).setData("mailto:".toUri())
                }
            }
    }

    override fun getSynchronousResult(
        context: Context,
        input: EmailData,
    ): SynchronousResult<Unit>? = null

    override fun parseResult(resultCode: Int, intent: Intent?): Unit = Unit

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
