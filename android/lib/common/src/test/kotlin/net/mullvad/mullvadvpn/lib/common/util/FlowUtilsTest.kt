package net.mullvad.mullvadvpn.lib.common.util

import io.mockk.coVerify
import io.mockk.mockk
import io.mockk.verify
import kotlin.coroutines.CoroutineContext
import kotlin.coroutines.cancellation.CancellationException
import kotlinx.coroutines.CoroutineName
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.awaitCancellation
import kotlinx.coroutines.cancelAndJoin
import kotlinx.coroutines.currentCoroutineContext
import kotlinx.coroutines.flow.flow
import kotlinx.coroutines.flow.flowOf
import kotlinx.coroutines.launch
import kotlinx.coroutines.test.runCurrent
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.withContext
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.assertNotNull

@OptIn(ExperimentalCoroutinesApi::class)
class FlowUtilsTest {

    @Test
    fun `when flow is subscribed, should call launch on start`() = runTest {
        // Arrange
        val mockFunction: () -> Unit = mockk(relaxed = true)
        val flow = flowOf(1, 2, 3).launchOnStart { mockFunction() }

        // Act
        flow.collect {}
        // Required to properly check if mockFunction was called.
        runCurrent()

        // Assert
        verify(exactly = 1) { mockFunction() }
    }

    @Test
    fun `launch on start should inherit the coroutine context from the collector`() = runTest {
        // Arrange
        var capturedContext: CoroutineContext? = null
        val flow = flowOf(1).launchOnStart { capturedContext = currentCoroutineContext() }

        // Act
        withContext(CoroutineName("CollectorContext")) { flow.collect {} }

        // Assert
        assertNotNull(capturedContext)
        assertEquals(
            "CollectorContext",
            capturedContext[CoroutineName]?.name,
        )
    }

    @Test
    fun `launch on start action should be cancelled when collector is cancelled`() = runTest {
        // Arrange
        var wasActionCancelled = false
        val flow =
            flow<Int> { awaitCancellation() }
                .launchOnStart {
                    try {
                        awaitCancellation()
                    } catch (e: CancellationException) {
                        wasActionCancelled = true
                        throw e
                    }
                }

        // Act
        val collectionJob = launch { flow.collect {} }
        runCurrent()
        collectionJob.cancelAndJoin()

        // Assert
        assertTrue(wasActionCancelled)
    }

    @Test
    fun `launch on start should be executed once per collector`() = runTest {
        // Arrange
        val mockFunction: suspend () -> Unit = mockk(relaxed = true)
        val flow = flowOf(1).launchOnStart { mockFunction() }

        // Act
        flow.collect {}
        flow.collect {}
        // Required to properly check if mockFunction was called.
        runCurrent()

        // Assert
        coVerify(exactly = 2) { mockFunction() }
    }
}
