package net.mullvad.mullvadvpn.service.endpoint

import java.net.InetAddress
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.channels.ClosedReceiveChannelException
import kotlinx.coroutines.channels.actor
import kotlinx.coroutines.channels.sendBlocking
import net.mullvad.mullvadvpn.ipc.Request
import net.mullvad.mullvadvpn.model.DnsOptions

class CustomDns(private val endpoint: ServiceEndpoint) {
    private sealed class Command {
        class AddDnsServer(val server: InetAddress) : Command()
        class RemoveDnsServer(val server: InetAddress) : Command()
        class ReplaceDnsServer(val oldServer: InetAddress, val newServer: InetAddress) : Command()
        class SetEnabled(val enabled: Boolean) : Command()
    }

    private val commandChannel = spawnActor()
    private val dnsServers = ArrayList<InetAddress>()

    private val daemon
        get() = endpoint.intermittentDaemon

    private var enabled = false

    init {
        endpoint.settingsListener.dnsOptionsNotifier.subscribe(this) { maybeDnsOptions ->
            maybeDnsOptions?.let { dnsOptions ->
                enabled = dnsOptions.custom
                dnsServers.clear()
                dnsServers.addAll(dnsOptions.addresses)
            }
        }

        endpoint.dispatcher.apply {
            registerHandler(Request.AddCustomDnsServer::class) { request ->
                commandChannel.sendBlocking(Command.AddDnsServer(request.address))
            }

            registerHandler(Request.RemoveCustomDnsServer::class) { request ->
                commandChannel.sendBlocking(Command.RemoveDnsServer(request.address))
            }

            registerHandler(Request.ReplaceCustomDnsServer::class) { request ->
                commandChannel.sendBlocking(
                    Command.ReplaceDnsServer(request.oldAddress, request.newAddress)
                )
            }

            registerHandler(Request.SetEnableCustomDns::class) { request ->
                commandChannel.sendBlocking(Command.SetEnabled(request.enable))
            }
        }
    }

    fun onDestroy() {
        endpoint.settingsListener.dnsOptionsNotifier.unsubscribe(this)
        commandChannel.close()
    }

    private fun spawnActor() = GlobalScope.actor<Command>(Dispatchers.Default, Channel.UNLIMITED) {
        try {
            while (true) {
                val command = channel.receive()

                when (command) {
                    is Command.AddDnsServer -> doAddDnsServer(command.server)
                    is Command.RemoveDnsServer -> doRemoveDnsServer(command.server)
                    is Command.ReplaceDnsServer -> {
                        doReplaceDnsServer(command.oldServer, command.newServer)
                    }
                    is Command.SetEnabled -> changeDnsOptions(command.enabled, dnsServers)
                }
            }
        } catch (exception: ClosedReceiveChannelException) {
            // Closed sender, so stop the actor
        }
    }

    private suspend fun doAddDnsServer(server: InetAddress) {
        if (!dnsServers.contains(server)) {
            dnsServers.add(server)
            changeDnsOptions(enabled, dnsServers)
        }
    }

    private suspend fun doReplaceDnsServer(oldServer: InetAddress, newServer: InetAddress) {
        if (oldServer != newServer && !dnsServers.contains(newServer)) {
            val index = dnsServers.indexOf(oldServer)

            if (index >= 0) {
                dnsServers.removeAt(index)
                dnsServers.add(index, newServer)
                changeDnsOptions(enabled, dnsServers)
            }
        }
    }

    private suspend fun doRemoveDnsServer(server: InetAddress) {
        if (dnsServers.remove(server)) {
            changeDnsOptions(enabled, dnsServers)
        }
    }

    private suspend fun changeDnsOptions(enable: Boolean, dnsServers: ArrayList<InetAddress>) {
        val options = DnsOptions(enable, dnsServers)

        daemon.await().setDnsOptions(options)
    }
}
