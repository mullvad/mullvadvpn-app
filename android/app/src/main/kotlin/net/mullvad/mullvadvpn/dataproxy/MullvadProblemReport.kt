package net.mullvad.mullvadvpn.dataproxy

import java.io.File
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.Deferred
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.async
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.channels.ClosedReceiveChannelException
import kotlinx.coroutines.channels.actor
import kotlinx.coroutines.channels.trySendBlocking

const val PROBLEM_REPORT_FILE = "problem_report.txt"

class MullvadProblemReport {
    private sealed class Command {
        class Collect() : Command()
        class Load(val logs: CompletableDeferred<String>) : Command()
        class Send(val result: CompletableDeferred<Boolean>) : Command()
        class Delete() : Command()
    }

    val logDirectory = CompletableDeferred<File>()
    val cacheDirectory = CompletableDeferred<File>()

    private val commandChannel = spawnActor()

    private val problemReportPath = GlobalScope.async(Dispatchers.Default) {
        File(logDirectory.await(), PROBLEM_REPORT_FILE)
    }

    private var isCollected = false

    var confirmNoEmail: CompletableDeferred<Boolean>? = null

    var userEmail = ""
    var userMessage = ""

    init {
        System.loadLibrary("mullvad_jni")
    }

    fun collect() {
        commandChannel.trySendBlocking(Command.Collect())
    }

    suspend fun load(): String {
        val logs = CompletableDeferred<String>()

        commandChannel.send(Command.Load(logs))

        return logs.await()
    }

    fun send(): Deferred<Boolean> {
        val result = CompletableDeferred<Boolean>()

        commandChannel.trySendBlocking(Command.Send(result))

        return result
    }

    fun deleteReportFile() {
        commandChannel.trySendBlocking(Command.Delete())
    }

    private fun spawnActor() = GlobalScope.actor<Command>(Dispatchers.Default, Channel.UNLIMITED) {
        try {
            while (true) {
                val command = channel.receive()

                when (command) {
                    is Command.Collect -> doCollect()
                    is Command.Load -> command.logs.complete(doLoad())
                    is Command.Send -> command.result.complete(doSend())
                    is Command.Delete -> doDelete()
                }
            }
        } catch (exception: ClosedReceiveChannelException) {
        }
    }

    private suspend fun doCollect() {
        val logDirectoryPath = logDirectory.await().absolutePath
        val reportPath = problemReportPath.await().absolutePath

        doDelete()

        isCollected = collectReport(logDirectoryPath, reportPath)
    }

    private suspend fun doLoad(): String {
        if (!isCollected) {
            doCollect()
        }

        if (isCollected) {
            return problemReportPath.await().readText()
        } else {
            return "Failed to collect logs for problem report"
        }
    }

    private suspend fun doSend(): Boolean {
        if (!isCollected) {
            doCollect()
        }

        val result = isCollected &&
            sendProblemReport(
                userEmail,
                userMessage,
                problemReportPath.await().absolutePath,
                cacheDirectory.await().absolutePath
            )

        if (result) {
            doDelete()
        }

        return result
    }

    private suspend fun doDelete() {
        problemReportPath.await().delete()
        isCollected = false
    }

    private external fun collectReport(logDirectory: String, reportPath: String): Boolean
    private external fun sendProblemReport(
        userEmail: String,
        userMessage: String,
        reportPath: String,
        cacheDirectory: String
    ): Boolean
}
