package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.viewModelScope
import app.cash.turbine.test
import io.mockk.MockKAnnotations
import io.mockk.coEvery
import io.mockk.impl.annotations.MockK
import kotlin.test.assertEquals
import kotlinx.coroutines.cancel
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.dataproxy.MullvadProblemReport
import net.mullvad.mullvadvpn.dataproxy.SendProblemReportResult
import net.mullvad.mullvadvpn.lib.common.test.TestCoroutineRule
import org.junit.After
import org.junit.Before
import org.junit.Rule
import org.junit.Test

class ReportProblemModelTest {
    @get:Rule val testCoroutineRule = TestCoroutineRule()

    @MockK private lateinit var mockMullvadProblemReport: MullvadProblemReport

    private lateinit var viewModel: ReportProblemViewModel

    @Before
    fun setUp() {
        MockKAnnotations.init(this)
        coEvery { mockMullvadProblemReport.collectLogs() } returns true
        viewModel = ReportProblemViewModel(mockMullvadProblemReport)
    }

    @After
    fun tearDown() {
        viewModel.viewModelScope.coroutineContext.cancel()
    }

    @Test
    fun sendReportFailedToCollectLogs() = runTest {
        // Arrange
        coEvery { mockMullvadProblemReport.sendReport(any()) } returns
            SendProblemReportResult.Error.CollectLog
        val email = "my@email.com"

        // Act, Assert
        viewModel.uiState.test {
            assertEquals(null, awaitItem().sendingState)
            viewModel.sendReport(email, "My description")
            assertEquals(SendingReportUiState.Sending, awaitItem().sendingState)
            assertEquals(
                SendingReportUiState.Error(SendProblemReportResult.Error.CollectLog),
                awaitItem().sendingState
            )
        }
    }

    @Test
    fun sendReportFailedToSendReport() = runTest {
        // Arrange
        coEvery { mockMullvadProblemReport.sendReport(any()) } returns
            SendProblemReportResult.Error.SendReport
        val email = "my@email.com"

        // Act, Assert
        viewModel.uiState.test {
            assertEquals(null, awaitItem().sendingState)
            viewModel.sendReport(email, "My description")
            assertEquals(SendingReportUiState.Sending, awaitItem().sendingState)
            assertEquals(
                SendingReportUiState.Error(SendProblemReportResult.Error.SendReport),
                awaitItem().sendingState
            )
        }
    }

    @Test
    fun sendReportWithoutEmailSuccessfully() = runTest {
        // Arrange
        coEvery { mockMullvadProblemReport.sendReport(any()) } returns
            SendProblemReportResult.Success
        val email = ""

        // Act, Assert
        viewModel.uiState.test {
            assertEquals(ReportProblemUiState(false, null), awaitItem())
            viewModel.sendReport(email, "My description")
            assertEquals(ReportProblemUiState(true, null), awaitItem())
            viewModel.sendReport(email, "My description")
            assertEquals(ReportProblemUiState(false, SendingReportUiState.Sending), awaitItem())
            assertEquals(
                ReportProblemUiState(false, SendingReportUiState.Success(null)),
                awaitItem()
            )
        }
    }

    @Test
    fun sendReportSuccessfully() = runTest {
        // Arrange
        coEvery { mockMullvadProblemReport.collectLogs() } returns true
        coEvery { mockMullvadProblemReport.sendReport(any()) } returns
            SendProblemReportResult.Success
        val email = "my@email.com"

        // Act, Assert
        viewModel.uiState.test {
            assertEquals(awaitItem(), ReportProblemUiState(false, null))
            viewModel.sendReport(email, "My description")

            assertEquals(awaitItem(), ReportProblemUiState(false, SendingReportUiState.Sending))
            assertEquals(
                awaitItem(),
                ReportProblemUiState(false, SendingReportUiState.Success(email))
            )
        }
    }
}
