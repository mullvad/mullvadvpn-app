package net.mullvad.mullvadvpn.service.endpoint

import java.net.InetAddress
import net.mullvad.mullvadvpn.ipc.Request
import net.mullvad.mullvadvpn.model.DnsOptions
import net.mullvad.mullvadvpn.service.endpoint.CustomDns.Command

class CustomDns(private val endpoint: ServiceEndpoint) : Actor<Command>() {
    sealed class Command {
        class AddDnsServer(val server: InetAddress) : Command()
        class RemoveDnsServer(val server: InetAddress) : Command()
        class ReplaceDnsServer(val oldServer: InetAddress, val newServer: InetAddress) : Command()
        class SetEnabled(val enabled: Boolean) : Command()
    }

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

        endpoint.dispatcher.run {
            registerHandler(Request.AddCustomDnsServer::class) { request ->
                sendBlocking(Command.AddDnsServer(request.address))
            }

            registerHandler(Request.RemoveCustomDnsServer::class) { request ->
                sendBlocking(Command.RemoveDnsServer(request.address))
            }

            registerHandler(Request.ReplaceCustomDnsServer::class) { request ->
                sendBlocking(Command.ReplaceDnsServer(request.oldAddress, request.newAddress))
            }

            registerHandler(Request.SetEnableCustomDns::class) { request ->
                sendBlocking(Command.SetEnabled(request.enable))
            }
        }
    }

    fun onDestroy() {
        endpoint.settingsListener.dnsOptionsNotifier.unsubscribe(this)
        closeActor()
    }

    override suspend fun onNewCommand(command: Command) = when (command) {
        is Command.AddDnsServer -> doAddDnsServer(command.server)
        is Command.RemoveDnsServer -> doRemoveDnsServer(command.server)
        is Command.ReplaceDnsServer -> doReplaceDnsServer(command.oldServer, command.newServer)
        is Command.SetEnabled -> changeDnsOptions(command.enabled)
    }

    private suspend fun doAddDnsServer(server: InetAddress) {
        if (!dnsServers.contains(server)) {
            dnsServers.add(server)
            changeDnsOptions(enabled)
        }
    }

    private suspend fun doReplaceDnsServer(oldServer: InetAddress, newServer: InetAddress) {
        if (oldServer != newServer && !dnsServers.contains(newServer)) {
            dnsServers.indexOf(oldServer).takeIf { it >= 0 }?.let { index ->
                dnsServers[index] = newServer
                changeDnsOptions(enabled)
            }
        }
    }

    private suspend fun doRemoveDnsServer(server: InetAddress) {
        if (dnsServers.remove(server)) {
            changeDnsOptions(enabled)
        }
    }

    private suspend fun changeDnsOptions(enable: Boolean) =
        daemon.await().setDnsOptions(DnsOptions(enable, dnsServers))
}
