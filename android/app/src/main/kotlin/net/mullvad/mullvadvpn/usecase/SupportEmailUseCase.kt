package net.mullvad.mullvadvpn.usecase

import android.content.Context
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.dataproxy.MullvadProblemReport
import net.mullvad.mullvadvpn.lib.model.BuildVersion

class SupportEmailUseCase(
    private val context: Context,
    private val mullvadProblemReport: MullvadProblemReport,
    private val buildVersion: BuildVersion,
) {
    suspend operator fun invoke(message: String? = null): SupportMail {
        return SupportMail(
            address = context.getString(R.string.support_email),
            subject = subject(),
            message = message,
            logs = mullvadProblemReport.readLogs(),
        )
    }

    // This is an approximation of the subject line that is used when a user contacts support
    // through the app
    private fun subject(): String {
        return context.getString(
            R.string.support_email_subject,
            buildVersion.name,
            android.os.Build.VERSION.RELEASE,
            android.os.Build.VERSION.SDK_INT,
            android.os.Build.MANUFACTURER,
            android.os.Build.MODEL,
        )
    }
}

data class SupportMail(
    val address: String,
    val subject: String,
    val message: String?,
    val logs: List<String>,
)
