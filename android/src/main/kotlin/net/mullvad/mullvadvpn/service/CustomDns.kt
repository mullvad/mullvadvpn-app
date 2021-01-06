package net.mullvad.mullvadvpn.service

import java.net.InetAddress
import java.util.ArrayList
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.channels.Channel
import kotlinx.coroutines.channels.ClosedReceiveChannelException
import kotlinx.coroutines.channels.actor
import kotlinx.coroutines.channels.sendBlocking
import net.mullvad.mullvadvpn.model.DnsOptions

class CustomDns(val daemon: MullvadDaemon, val settingsListener: SettingsListener) {
    private sealed class Command {
        class AddDnsServer(val server: InetAddress) : Command()
        class RemoveDnsServer(val server: InetAddress) : Command()
        class ReplaceDnsServer(val oldServer: InetAddress, val newServer: InetAddress) : Command()
        class SetEnabled(val enabled: Boolean) : Command()
    }

    private val commandChannel = spawnActor()

    private var dnsServers = ArrayList<InetAddress>()
    private var enabled = false

    init {
        settingsListener.dnsOptionsNotifier.subscribe(this) { maybeDnsOptions ->
            maybeDnsOptions?.let { dnsOptions ->
                enabled = dnsOptions.custom
                dnsServers = ArrayList(dnsOptions.addresses)
            }
        }
    }

    fun onDestroy() {
        settingsListener.dnsOptionsNotifier.unsubscribe(this)
        commandChannel.close()
    }

    fun addDnsServer(server: InetAddress) {
        commandChannel.sendBlocking(Command.AddDnsServer(server))
    }

    fun replaceDnsServer(oldServer: InetAddress, newServer: InetAddress) {
        commandChannel.sendBlocking(Command.ReplaceDnsServer(oldServer, newServer))
    }

    fun removeDnsServer(server: InetAddress) {
        commandChannel.sendBlocking(Command.RemoveDnsServer(server))
    }

    fun setEnabled(enabled: Boolean) {
        commandChannel.sendBlocking(Command.SetEnabled(enabled))
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

    private fun doAddDnsServer(server: InetAddress) {
        if (!dnsServers.contains(server)) {
            dnsServers.add(server)
            changeDnsOptions(enabled, dnsServers)
        }
    }

    private fun doReplaceDnsServer(oldServer: InetAddress, newServer: InetAddress) {
        if (oldServer != newServer && !dnsServers.contains(newServer)) {
            val index = dnsServers.indexOf(oldServer)

            if (index >= 0) {
                dnsServers.removeAt(index)
                dnsServers.add(index, newServer)
                changeDnsOptions(enabled, dnsServers)
            }
        }
    }

    private fun doRemoveDnsServer(server: InetAddress) {
        if (dnsServers.remove(server)) {
            changeDnsOptions(enabled, dnsServers)
        }
    }

    private fun changeDnsOptions(enable: Boolean, dnsServers: ArrayList<InetAddress>) {
        val options = DnsOptions(enable, dnsServers)

        daemon.setDnsOptions(options)
    }
}
