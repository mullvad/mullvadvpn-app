package net.mullvad.mullvadvpn.viewmodel

import androidx.lifecycle.viewModelScope
import app.cash.turbine.test
import io.mockk.MockKAnnotations
import io.mockk.coEvery
import io.mockk.impl.annotations.MockK
import io.mockk.verify
import kotlin.test.assertEquals
import kotlinx.coroutines.cancel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.test.runTest
import net.mullvad.mullvadvpn.dataproxy.MullvadProblemReport
import net.mullvad.mullvadvpn.dataproxy.SendProblemReportResult
import net.mullvad.mullvadvpn.dataproxy.UserReport
import net.mullvad.mullvadvpn.lib.common.test.TestCoroutineRule
import net.mullvad.mullvadvpn.repository.ProblemReportRepository
import org.junit.After
import org.junit.Before
import org.junit.Rule
import org.junit.Test

class ReportProblemViewModelTest {
    @get:Rule val testCoroutineRule = TestCoroutineRule()

    @MockK private lateinit var mockMullvadProblemReport: MullvadProblemReport

    @MockK(relaxed = true) private lateinit var mockProblemReportRepository: ProblemReportRepository

    private val problemReportFlow = MutableStateFlow(UserReport("", ""))

    private lateinit var viewModel: ReportProblemViewModel

    @Before
    fun setUp() {
        MockKAnnotations.init(this)
        coEvery { mockMullvadProblemReport.collectLogs() } returns true
        coEvery { mockProblemReportRepository.problemReport } returns problemReportFlow
        viewModel = ReportProblemViewModel(mockMullvadProblemReport, mockProblemReportRepository)
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
        val description = "My description"

        coEvery { mockProblemReportRepository.setDescription(any()) } answers
            {
                problemReportFlow.value = problemReportFlow.value.copy(description = arg(0))
            }

        // Act, Assert
        viewModel.uiState.test {
            assertEquals(ReportProblemUiState(), awaitItem())
            viewModel.updateDescription(description)
            assertEquals(ReportProblemUiState(description = description), awaitItem())

            viewModel.sendReport(email, description, true)
            assertEquals(
                ReportProblemUiState(SendingReportUiState.Sending, email, description),
                awaitItem()
            )
            assertEquals(
                ReportProblemUiState(
                    SendingReportUiState.Success(null),
                    "",
                    "",
                ),
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
        val description = "My description"

        // This might look a bit weird, and is not optimal. An alternative would be to use the real
        // ProblemReportRepository, but that would complicate the other tests. This is a compromise.
        coEvery { mockProblemReportRepository.setEmail(any()) } answers
            {
                problemReportFlow.value = problemReportFlow.value.copy(email = arg(0))
            }
        coEvery { mockProblemReportRepository.setDescription(any()) } answers
            {
                problemReportFlow.value = problemReportFlow.value.copy(description = arg(0))
            }

        // Act, Assert
        viewModel.uiState.test {
            assertEquals(awaitItem(), ReportProblemUiState(null, "", ""))
            viewModel.updateEmail(email)
            awaitItem()
            viewModel.updateDescription(description)
            awaitItem()

            viewModel.sendReport(email, description)

            assertEquals(
                ReportProblemUiState(
                    SendingReportUiState.Sending,
                    email,
                    description,
                ),
                awaitItem()
            )
            assertEquals(
                ReportProblemUiState(
                    SendingReportUiState.Success(email),
                    "",
                    "",
                ),
                awaitItem()
            )
        }
    }

    @Test
    fun testUpdateEmail() = runTest {
        // Arrange
        val email = "my@email.com"

        // Act
        viewModel.updateEmail(email)

        // Assert
        verify { mockProblemReportRepository.setEmail(email) }
    }

    @Test
    fun testUpdateDescription() = runTest {
        // Arrange
        val description = "My description"

        // Act
        viewModel.updateDescription(description)

        // Assert
        verify { mockProblemReportRepository.setDescription(description) }
    }
}
