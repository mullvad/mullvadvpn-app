package net.mullvad.mullvadvpn.lib.usecase

import android.content.Context
import android.os.Build
import net.mullvad.mullvadvpn.lib.model.BuildVersion
import net.mullvad.mullvadvpn.lib.repository.ProblemReportRepository

class SupportEmailUseCase(
    private val context: Context,
    private val problemReportRepository: ProblemReportRepository,
    private val buildVersion: BuildVersion,
) {
    suspend operator fun invoke(message: String? = null): SupportMail {
        return SupportMail(
            address = context.getString(R.string.support_email),
            subject = subject(),
            message = message,
            logs = problemReportRepository.readLogs(),
        )
    }

    // This is an approximation of the subject line that is used when a user contacts support
    // through the app
    private fun subject(): String {
        return context.getString(
            R.string.support_email_subject,
            buildVersion.name,
            Build.VERSION.RELEASE,
            Build.VERSION.SDK_INT,
            Build.MANUFACTURER,
            Build.MODEL,
        )
    }
}

data class SupportMail(
    val address: String,
    val subject: String,
    val message: String?,
    val logs: List<String>,
)
