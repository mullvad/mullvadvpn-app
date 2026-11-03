package net.mullvad.mullvadvpn.lib.grpc.util

import com.squareup.wire.GrpcCall
import io.mockk.awaits
import io.mockk.coEvery
import io.mockk.coVerify
import io.mockk.every
import io.mockk.just
import io.mockk.mockk
import java.io.IOException
import kotlin.test.assertFailsWith
import kotlin.time.Duration.Companion.milliseconds
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.TimeoutCancellationException
import kotlinx.coroutines.async
import kotlinx.coroutines.cancelAndJoin
import kotlinx.coroutines.launch
import kotlinx.coroutines.test.advanceTimeBy
import kotlinx.coroutines.test.runCurrent
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.withTimeout
import org.junit.jupiter.api.Assertions.assertEquals
import org.junit.jupiter.api.Assertions.assertTrue
import org.junit.jupiter.api.Test

@OptIn(ExperimentalCoroutinesApi::class)
class GrpcCallTest {
    @Test
    fun `when a call is successful it returns the result and the call is only called once`() =
        runTest {
            val mockCall = mockk<GrpcCall<Unit, String>>()
            every { mockCall.clone() } returns mockCall
            coEvery { mockCall.execute(Unit) } returns "result"

            assertEquals("result", mockCall.executeWithRetry(Unit, {}))
            coVerify(exactly = 1) { mockCall.execute(Unit) }
        }

    @Test
    fun `when a call fails due to LocalSocketException it is retried until successful`() = runTest {
        val mockCall = mockk<GrpcCall<Unit, String>>()
        val localSocketException = LocalSocketConnectException(Throwable())
        every { mockCall.clone() } returns mockCall
        coEvery { mockCall.execute(Unit) } throwsMany listOf(localSocketException) andThen "result"

        val result = async { mockCall.executeWithRetry(Unit, {}) }
        // Needs to be above the retry delay to ensure the retry is triggered.
        advanceTimeBy(150.milliseconds)
        runCurrent()

        assertEquals("result", result.await())
        coVerify(exactly = 2) { mockCall.execute(Unit) }
    }

    @Test
    fun `when a call fails due to other IOException it will never retry`() = runTest {
        val mockCall = mockk<GrpcCall<Unit, String>>()
        every { mockCall.clone() } returns mockCall
        coEvery { mockCall.execute(Unit) } throws IOException()

        assertFailsWith<IOException> { mockCall.executeWithRetry(Unit, {}) }
        coVerify(exactly = 1) { mockCall.execute(Unit) }
    }

    @Test
    fun `when cancelling a call it should stop retrying`() = runTest {
        val mockCall = mockk<GrpcCall<Unit, String>>()
        every { mockCall.clone() } returns mockCall
        coEvery { mockCall.execute(Unit) } throws LocalSocketConnectException(Throwable())

        val job = launch { mockCall.executeWithRetry(Unit, {}) }
        // Allow for a few attempts to be made before cancelling.
        advanceTimeBy(300.milliseconds)
        job.cancelAndJoin()

        assertTrue(job.isCancelled)
        // Moving time forward must not trigger further attempts after cancellation.
        advanceTimeBy(1000.milliseconds)
        runCurrent()
        coVerify(exactly = 3) { mockCall.execute(Unit) }
    }

    @Test
    fun `when wrapping a call in withTimeout it should cancel properly`() = runTest {
        val mockCall = mockk<GrpcCall<Unit, String>>()
        every { mockCall.clone() } returns mockCall
        coEvery { mockCall.execute(Unit) } just awaits

        assertFailsWith<TimeoutCancellationException> {
            withTimeout(300.milliseconds) { mockCall.executeWithRetry(Unit, {}) }
        }
    }

    @Test
    fun `when keepRetrying turns false the connection failure is thrown`() = runTest {
        val mockCall = mockk<GrpcCall<Unit, String>>()
        every { mockCall.clone() } returns mockCall
        coEvery { mockCall.execute(Unit) } throws LocalSocketConnectException(Throwable())
        var keepRetrying = true

        val result = async { runCatching { mockCall.executeWithRetry(Unit) { keepRetrying } } }
        advanceTimeBy(1_000.milliseconds)
        assertTrue(result.isActive)

        keepRetrying = false
        advanceTimeBy(1_000.milliseconds)

        assertTrue(result.await().exceptionOrNull() is LocalSocketConnectException)
    }
}
