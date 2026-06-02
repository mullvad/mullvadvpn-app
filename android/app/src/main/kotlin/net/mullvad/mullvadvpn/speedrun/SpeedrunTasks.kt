package net.mullvad.mullvadvpn.speedrun

import net.mullvad.mullvadvpn.BuildConfig
import net.mullvad.mullvadvpn.lib.model.DeviceState
import net.mullvad.mullvadvpn.lib.model.GeoIpLocation
import net.mullvad.mullvadvpn.lib.model.ObfuscationMode
import net.mullvad.mullvadvpn.lib.model.Port
import net.mullvad.mullvadvpn.lib.model.Settings
import net.mullvad.mullvadvpn.lib.model.TunnelState

/**
 * TEMPORARY, non-production "team fair speed-run competition" feature.
 *
 * Everything lives under this single package so it can be deleted in one go. Task completion is
 * detected purely by observing the daemon state the app already streams over gRPC (tunnel state,
 * device state, settings) — there is no UI automation and no new dependency.
 *
 * Edit the constants below to configure a run.
 */
object SpeedrunConfig {
    /** Relay to connect to in step 2 and to use as the multihop entry in step 4. */
    const val ENTRY_RELAY = "nl-ams-wg-008"

    /** Multihop exit relay for step 4. */
    const val EXIT_RELAY = "se-got-wg-007"

    /**
     * The account players must log in to. Injected from the Gradle property
     * `mullvad.app.config.speedrun.accountNumber` (a placeholder unless a dev overrides it). Login
     * detection requires an exact match (spaces are ignored).
     */
    val ACCOUNT_NUMBER = BuildConfig.SPEEDRUN_ACCOUNT_NUMBER

    /** Obfuscation port required by the LWO step. */
    const val LWO_PORT = 13337

    /**
     * Footprint control. `true` (default) shows the current task under the timer; `false` shows
     * only the timer. There is intentionally no "next task" preview and no full checklist.
     */
    const val SHOW_CURRENT_TASK = true
}

/** Digits of [SpeedrunConfig.ACCOUNT_NUMBER] used for the login match (grouping spaces removed). */
private val ACCOUNT_DIGITS = SpeedrunConfig.ACCOUNT_NUMBER.filter(Char::isDigit)

/** Latest values of the three daemon flows we care about, combined into one snapshot. */
data class DaemonSnapshot(
    val deviceState: DeviceState?,
    val tunnelState: TunnelState?,
    val settings: Settings?,
)

/** A single ordered step. [isComplete] is evaluated only while it is the *current* step. */
data class SpeedrunTask(val title: String, val isComplete: (DaemonSnapshot) -> Boolean)

private fun DaemonSnapshot.connectedLocation(): GeoIpLocation? =
    (tunnelState as? TunnelState.Connected)?.location

/**
 * The ordered task list. The engine only ever evaluates the current task, which is what makes
 * "reach the same state more than once" (e.g. LAN sharing off -> on -> off) work correctly and
 * enforces that steps are done in order.
 */
val SPEEDRUN_TASKS: List<SpeedrunTask> =
    listOf(
        SpeedrunTask("Log in to: ${SpeedrunConfig.ACCOUNT_NUMBER}") {
            (it.deviceState as? DeviceState.LoggedIn)?.accountNumber?.value == ACCOUNT_DIGITS
        },
        SpeedrunTask("Connect to ${SpeedrunConfig.ENTRY_RELAY}") {
            val location = it.connectedLocation()
            // Single-hop: the relay is the exit and there is no entry hop.
            location != null &&
                location.hostname == SpeedrunConfig.ENTRY_RELAY &&
                location.entryHostname == null
        },
        SpeedrunTask("Enable LWO with auto port") {
            val obfuscation = it.settings?.obfuscationSettings
            // Auto port = no specific port constraint (Constraint.Any).
            obfuscation != null &&
                obfuscation.selectedObfuscationMode == ObfuscationMode.Lwo &&
                obfuscation.lwo.port.getOrNull() == null
        },
        SpeedrunTask(
            "Multihop ${SpeedrunConfig.ENTRY_RELAY} (entry) → ${SpeedrunConfig.EXIT_RELAY} (exit)"
        ) {
            val location = it.connectedLocation()
            location != null &&
                location.entryHostname == SpeedrunConfig.ENTRY_RELAY &&
                location.hostname == SpeedrunConfig.EXIT_RELAY
        },
        SpeedrunTask("Enable all DNS blockers except ads") {
            val blockers = it.settings?.tunnelOptions?.dnsOptions?.defaultOptions
            blockers != null &&
                !blockers.blockAds &&
                blockers.blockTrackers &&
                blockers.blockMalware &&
                blockers.blockAdultContent &&
                blockers.blockGambling &&
                blockers.blockSocialMedia
        },
        SpeedrunTask("Switch LWO port to ${SpeedrunConfig.LWO_PORT}") {
            val obfuscation = it.settings?.obfuscationSettings
            obfuscation != null &&
                obfuscation.selectedObfuscationMode == ObfuscationMode.Lwo &&
                obfuscation.lwo.port.getOrNull() == Port(SpeedrunConfig.LWO_PORT)
        },
        SpeedrunTask("Disable DNS blockers") {
            it.settings?.tunnelOptions?.dnsOptions?.defaultOptions?.isAnyBlockerEnabled == false
        },
        SpeedrunTask("Log out") { it.deviceState is DeviceState.LoggedOut },
    )

/**
 * The fair starting point. The Start button is only offered when the live state matches this, so a
 * player cannot pre-stage steps before starting the timer. A freshly cleared app satisfies it.
 */
fun isCleanBaseline(snapshot: DaemonSnapshot): Boolean {
    val settings = snapshot.settings ?: return false
    val loggedOut =
        snapshot.deviceState is DeviceState.LoggedOut || snapshot.deviceState is DeviceState.Revoked
    val disconnected = snapshot.tunnelState is TunnelState.Disconnected
    val noBlockers = !settings.tunnelOptions.dnsOptions.defaultOptions.isAnyBlockerEnabled
    val noMultihop = !settings.relaySettings.relayConstraints.wireguardConstraints.isMultihopEnabled
    val noLwo = settings.obfuscationSettings.selectedObfuscationMode != ObfuscationMode.Lwo
    return loggedOut && disconnected && noBlockers && noMultihop && noLwo
}

/** True once we have received enough data from the daemon to trust [isCleanBaseline]. */
fun isBaselineKnown(snapshot: DaemonSnapshot): Boolean =
    snapshot.deviceState != null && snapshot.settings != null && snapshot.tunnelState != null
