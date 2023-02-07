package net.mullvad.mullvadvpn.service.endpoint

import java.net.InetAddress
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.channels.ClosedReceiveChannelException
import kotlinx.coroutines.channels.actor
import kotlinx.coroutines.channels.trySendBlocking
import net.mullvad.mullvadvpn.ipc.Request
import net.mullvad.mullvadvpn.model.CustomDnsOptions
import net.mullvad.mullvadvpn.model.DefaultDnsOptions
import net.mullvad.mullvadvpn.model.DnsOptions
import net.mullvad.mullvadvpn.model.DnsState

class CustomDns(private val endpoint: ServiceEndpoint) {
    private sealed class Command {
        @Deprecated("Use SetDnsOptions")
        class AddDnsServer(val server: InetAddress) : Command()
        @Deprecated("Use SetDnsOptions")
        class RemoveDnsServer(val server: InetAddress) : Command()
        @Deprecated("Use SetDnsOptions")
        class ReplaceDnsServer(val oldServer: InetAddress, val newServer: InetAddress) : Command()
        @Deprecated("Use SetDnsOptions")
        class SetEnabled(val enabled: Boolean) : Command()

        class SetDnsOptions(val dnsOptions: DnsOptions) : Command()
    }

    private val commandChannel = spawnActor()
    private val dnsServers = ArrayList<InetAddress>()

    private val daemon
        get() = endpoint.intermittentDaemon

    private var enabled = false

    init {
        endpoint.settingsListener.dnsOptionsNotifier.subscribe(this) { maybeDnsOptions ->
            maybeDnsOptions?.let { dnsOptions ->
                enabled = dnsOptions.state == DnsState.Custom
                dnsServers.clear()
                dnsServers.addAll(dnsOptions.customOptions.addresses)
            }
        }

        endpoint.dispatcher.apply {
            registerHandler(Request.AddCustomDnsServer::class) { request ->
                commandChannel.trySendBlocking(Command.AddDnsServer(request.address))
            }

            registerHandler(Request.RemoveCustomDnsServer::class) { request ->
                commandChannel.trySendBlocking(Command.RemoveDnsServer(request.address))
            }

            registerHandler(Request.ReplaceCustomDnsServer::class) { request ->
                commandChannel.trySendBlocking(
                    Command.ReplaceDnsServer(request.oldAddress, request.newAddress)
                )
            }

            registerHandler(Request.SetEnableCustomDns::class) { request ->
                commandChannel.trySendBlocking(Command.SetEnabled(request.enable))
            }

            registerHandler(Request.SetDnsOptions::class) { request ->
                commandChannel.trySendBlocking(Command.SetDnsOptions(request.dnsOptions))
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
                    is Command.SetEnabled -> changeDnsOptions(command.enabled)
                    is Command.SetDnsOptions -> setDnsOptions(command.dnsOptions)
                }
            }
        } catch (exception: ClosedReceiveChannelException) {
            // Closed sender, so stop the actor
        }
    }

    private suspend fun doAddDnsServer(server: InetAddress) {
        if (!dnsServers.contains(server)) {
            dnsServers.add(server)
            changeDnsOptions(enabled)
        }
    }

    private suspend fun doReplaceDnsServer(oldServer: InetAddress, newServer: InetAddress) {
        if (oldServer != newServer && !dnsServers.contains(newServer)) {
            val index = dnsServers.indexOf(oldServer)

            if (index >= 0) {
                dnsServers.removeAt(index)
                dnsServers.add(index, newServer)
                changeDnsOptions(enabled)
            }
        }
    }

    private suspend fun doRemoveDnsServer(server: InetAddress) {
        if (dnsServers.remove(server)) {
            changeDnsOptions(enabled)
        }
    }

    private suspend fun changeDnsOptions(enable: Boolean) {
        val options = DnsOptions(
            state = if (enable) DnsState.Custom else DnsState.Default,
            customOptions = CustomDnsOptions(dnsServers),
            defaultOptions = DefaultDnsOptions()
        )
        daemon.await().setDnsOptions(options)
    }

    private suspend fun setDnsOptions(dnsOptions: DnsOptions) {
        daemon.await().setDnsOptions(dnsOptions)
    }
}
