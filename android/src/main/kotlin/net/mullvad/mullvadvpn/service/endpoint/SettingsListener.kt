package net.mullvad.mullvadvpn.service.endpoint

import net.mullvad.mullvadvpn.ipc.Event
import net.mullvad.mullvadvpn.ipc.Request
import net.mullvad.mullvadvpn.model.DnsOptions
import net.mullvad.mullvadvpn.model.RelaySettings
import net.mullvad.mullvadvpn.model.Settings
import net.mullvad.mullvadvpn.service.MullvadDaemon
import net.mullvad.mullvadvpn.service.endpoint.SettingsListener.Command
import net.mullvad.talpid.util.EventNotifier

class SettingsListener(endpoint: ServiceEndpoint) : Actor<Command>() {
    sealed class Command {
        class SetAllowLan(val allow: Boolean) : Command()
        class SetAutoConnect(val autoConnect: Boolean) : Command()
        class SetWireGuardMtu(val mtu: Int?) : Command()
    }

    private val intermittentDaemon = endpoint.intermittentDaemon

    val accountNumberNotifier = EventNotifier<String?>(null)
    val dnsOptionsNotifier = EventNotifier<DnsOptions?>(null)
    val relaySettingsNotifier = EventNotifier<RelaySettings?>(null)
    private val settingsNotifier = EventNotifier<Settings?>(null)

    var settings by settingsNotifier.notifiable()
        private set

    init {
        intermittentDaemon.registerListener(this) { newDaemon ->
            if (newDaemon != null) {
                registerListener(newDaemon)
                fetchInitialSettings(newDaemon)
            }
        }

        settingsNotifier.subscribe(this) { settings ->
            endpoint.sendEvent(Event.SettingsUpdate(settings))
        }

        endpoint.dispatcher.run {
            registerHandler(Request.SetAllowLan::class) { request ->
                sendBlocking(Command.SetAllowLan(request.allow))
            }

            registerHandler(Request.SetAutoConnect::class) { request ->
                sendBlocking(Command.SetAutoConnect(request.autoConnect))
            }

            registerHandler(Request.SetWireGuardMtu::class) { request ->
                sendBlocking(Command.SetWireGuardMtu(request.mtu))
            }
        }
    }

    fun onDestroy() {
        closeActor()
        intermittentDaemon.unregisterListener(this)

        accountNumberNotifier.unsubscribeAll()
        dnsOptionsNotifier.unsubscribeAll()
        relaySettingsNotifier.unsubscribeAll()
        settingsNotifier.unsubscribeAll()
    }

    fun subscribe(id: Any, listener: (Settings) -> Unit) =
        settingsNotifier.subscribe(id) { maybeSettings ->
            maybeSettings?.run { listener(this) }
        }

    fun unsubscribe(id: Any) = settingsNotifier.unsubscribe(id)

    private fun registerListener(daemon: MullvadDaemon) =
        daemon.onSettingsChange.subscribe(this, ::handleNewSettings)

    private fun fetchInitialSettings(daemon: MullvadDaemon) =
        handleNewSettings(daemon.getSettings())

    private fun handleNewSettings(newSettings: Settings?) = newSettings?.let {
        synchronized(this) {
            if (settings?.accountToken != it.accountToken) {
                accountNumberNotifier.notify(it.accountToken)
            }

            if (settings?.tunnelOptions?.dnsOptions != it.tunnelOptions.dnsOptions) {
                dnsOptionsNotifier.notify(it.tunnelOptions.dnsOptions)
            }

            if (settings?.relaySettings != it.relaySettings) {
                relaySettingsNotifier.notify(it.relaySettings)
            }

            settings = it
        }
    }

    override suspend fun onNewCommand(command: Command) = when (command) {
        is Command.SetAllowLan -> intermittentDaemon.await().setAllowLan(command.allow)
        is Command.SetAutoConnect -> intermittentDaemon.await().setAutoConnect(command.autoConnect)
        is Command.SetWireGuardMtu -> intermittentDaemon.await().setWireguardMtu(command.mtu)
    }
}
