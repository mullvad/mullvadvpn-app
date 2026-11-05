// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import MullvadLogging
import NetworkExtension
import SwiftUI

struct MapView: View {
    @Environment(Navigation.self) private var navigation: Navigation
    let relays: RelayListLoader

    var body: some View {
        VStack {
            HeaderBar()
                .frame(minWidth: 0, maxWidth: .infinity, minHeight: 0, maxHeight: 100)
                .background(Color.mullvadDangerColor)
            Spacer()
            Text("Congratulations you are logged in !")

            Spacer()

            TunnelButton(relays: relays)

            Spacer()
        }
        .background(Color.mullvadBackground)
        .task { relays.updateIfEmpty() }
    }
}

/// Keeps at most one relay list update in flight, so that callers can await the running one
/// instead of starting a second.
@MainActor @Observable
final class RelayListLoader {
    private(set) var isUpdating = false

    @ObservationIgnored private let logger = Logger(label: "RelayListLoader")
    @ObservationIgnored private let updater: any RelayListUpdating
    @ObservationIgnored private var task: Task<Void, Never>?

    init(updater: any RelayListUpdating) {
        self.updater = updater
    }

    /// Fetches the relay list when nothing is cached, and does nothing otherwise.
    func updateIfEmpty() {
        guard updater.cachedRelayCount == 0 else { return }
        update()
    }

    /// Fetches the relay list, unless a fetch is already running.
    private func update() {
        guard task == nil else { return }
        isUpdating = true
        task = Task {
            do {
                try await updater.updateRelays()
            } catch {
                logger.error(error: error, message: "Failed to update the relay list.")
            }
            isUpdating = false
            task = nil
        }
    }

    /// Waits for a running fetch, and returns immediately when there is none.
    func waitForUpdate() async {
        await task?.value
    }
}

@MainActor @Observable
final class TunnelController {
    private(set) var status: NEVPNStatus = .invalid
    private(set) var lastError: Error?

    @ObservationIgnored private let providerBundleID = "net.mullvad.MullvadVPNTvOS.PacketTunnel"
    @ObservationIgnored private var manager: NETunnelProviderManager?
    @ObservationIgnored private var observer: NSObjectProtocol?

    init() {
        Task { _ = try? await existingManager() }
    }

    func toggle() async {
        lastError = nil
        do {
            switch status {
            case .connected, .connecting, .reasserting:
                manager?.connection.stopVPNTunnel()
            default:
                try await connect()
            }
        } catch {
            lastError = error
        }
    }

    private func connect() async throws {
        let m: NETunnelProviderManager
        if let existing = try await existingManager() {
            m = existing
        } else {
            m = NETunnelProviderManager()
            let proto = NETunnelProviderProtocol()
            proto.providerBundleIdentifier = providerBundleID
            proto.serverAddress = "127.0.0.1"
            m.protocolConfiguration = proto
            m.localizedDescription = "Mullvad VPN"
        }

        try await m.loadFromPreferences()
        m.isEnabled = true
        // Another VPN app enabling its config flips ours to disabled,
        // so check every time, not only on first install.
        try await m.saveToPreferences()  // first time: system "Add VPN Configurations" prompt
        try await m.loadFromPreferences()  // required after save, otherwise start fails with stale/invalid config

        attach(m)
        try m.connection.startVPNTunnel(options: nil)
        try await m.connection.fetchLastDisconnectError()
    }

    private func existingManager() async throws -> NETunnelProviderManager? {
        if let manager { return manager }
        let all = try await NETunnelProviderManager.loadAllFromPreferences()
        let found = all.first {
            ($0.protocolConfiguration as? NETunnelProviderProtocol)?.providerBundleIdentifier == providerBundleID
        }
        if let found { attach(found) }
        return found
    }

    private func attach(_ m: NETunnelProviderManager) {
        guard manager !== m else { return }
        manager = m
        status = m.connection.status
        if let observer { NotificationCenter.default.removeObserver(observer) }
        observer = NotificationCenter.default.addObserver(
            forName: .NEVPNStatusDidChange, object: m.connection, queue: .main
        ) { [weak self] note in
            let s = (note.object as? NEVPNConnection)?.status ?? .invalid
            MainActor.assumeIsolated { self?.status = s }
        }
    }
}

struct TunnelButton: View {
    let relays: RelayListLoader

    @State private var tunnel = TunnelController()

    var body: some View {
        VStack(spacing: 8) {
            Button(buttonTitle) {
                Task {
                    // Starting with an empty relay cache leaves the tunnel blocking all traffic,
                    // so let a fetch that is already running finish first. Deliberately not
                    // disabling the button while it runs: the press is what schedules the wait.
                    await relays.waitForUpdate()
                    await tunnel.toggle()
                }
            }
            .disabled(tunnel.status == .disconnecting)
            Text(relays.isUpdating ? "Fetching relays…" : statusText)
                .font(.caption).foregroundStyle(.secondary)
            if let err = tunnel.lastError {
                Text(err.localizedDescription).font(.caption).foregroundStyle(.red)
            }
        }
    }

    private var buttonTitle: String {
        switch tunnel.status {
        case .connected, .connecting, .reasserting: "Disconnect"
        default: "Connect"
        }
    }

    private var statusText: String {
        switch tunnel.status {
        case .invalid: "Not configured"
        case .disconnected: "Disconnected"
        case .connecting: "Connecting…"
        case .connected: "Connected"
        case .reasserting: "Reconnecting…"
        case .disconnecting: "Disconnecting…"
        @unknown default: "Unknown"
        }
    }
}

#Preview {
    MapView(relays: RelayListLoader(updater: RelayListUpdaterStub()))
        .environment(Navigation())
}
