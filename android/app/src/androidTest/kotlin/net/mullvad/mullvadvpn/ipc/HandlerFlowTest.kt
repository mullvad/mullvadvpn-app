package net.mullvad.mullvadvpn.ipc

import android.os.Bundle
import android.os.Looper
import android.os.Message
import android.os.Parcelable
import kotlin.test.assertEquals
import kotlinx.coroutines.flow.take
import kotlinx.coroutines.flow.toList
import kotlinx.coroutines.runBlocking
import kotlinx.parcelize.Parcelize
import org.junit.Test

class HandlerFlowTest {
    val looper by lazy { Looper.getMainLooper() }

    val handler: HandlerFlow<Data?> by lazy {
        HandlerFlow(looper) { message -> message.data.getParcelable(DATA_KEY) }
    }

    @Test
    fun test_message_extraction() {
        sendMessage(Data(1))
        sendMessage(Data(2))
        sendMessage(Data(3))

        val extractedData = runBlocking { handler.take(3).toList() }

        assertEquals(listOf(Data(1), Data(2), Data(3)), extractedData)
    }

    private fun sendMessage(messageData: Data) {
        val message =
            Message().apply { data = Bundle().apply { putParcelable(DATA_KEY, messageData) } }

        handler.handleMessage(message)
    }

    companion object {
        const val DATA_KEY = "data"

        @Parcelize data class Data(val id: Int) : Parcelable
    }
}
