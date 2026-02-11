package net.mullvad.mullvadvpn.feature.problemreport.impl

import androidx.compose.ui.tooling.preview.PreviewParameterProvider
import net.mullvad.mullvadvpn.lib.repository.SendProblemReportResult

class ReportProblemUiStatePreviewParameterProvider :
    PreviewParameterProvider<ReportProblemUiState> {
    override val values: Sequence<ReportProblemUiState>
        get() =
            sequenceOf(
                ReportProblemUiState(showIncludeAccountId = true),
                ReportProblemUiState(showIncludeAccountId = true, includeAccountId = true),
                ReportProblemUiState(
                    showIncludeAccountId = true,
                    includeAccountId = true,
                    showIncludeAccountWarningMessage = true,
                ),
                ReportProblemUiState(sendingState = SendingReportUiState.Sending),
                ReportProblemUiState(sendingState = SendingReportUiState.Success("email@mail.com")),
                ReportProblemUiState(
                    sendingState =
                        SendingReportUiState.Error(SendProblemReportResult.Error.CollectLog)
                ),
            )
}
