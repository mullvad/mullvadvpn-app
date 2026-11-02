// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import NetworkExtension
import SwiftUI

struct RootView: View {
    @Environment(Navigation.self) private var navigation: Navigation
    private var viewModel: any LoggingIn

    init(viewModel: any LoggingIn) {
        self.viewModel = viewModel
    }

    // Need to create an account
    // Need to login into an account
    var body: some View {
        VStack {
            HeaderBar()
                .frame(minWidth: 0, maxWidth: .infinity, minHeight: 0, maxHeight: 200)
            Spacer()

            LoginView(viewModel: viewModel)

            Spacer()

            TunnelButton()

            Spacer()

            RelayReloadButton()

            Spacer()

            CreateAccountPromptView()
                .background(Color.mullvadBackground)
                .frame(minWidth: 0, maxWidth: .infinity, minHeight: 0, maxHeight: 150)
        }
        .frame(minWidth: 0, maxWidth: .infinity, minHeight: 0, maxHeight: .infinity)
        .background(Color.mullvadBackground)
        .environment(navigation)
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
    @State private var tunnel = TunnelController()

    var body: some View {
        VStack(spacing: 8) {
            Button(buttonTitle) { Task { await tunnel.toggle() } }
                .disabled(tunnel.status == .disconnecting)
            Text(statusText).font(.caption).foregroundStyle(.secondary)
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

/// Refetches the relay list on demand.
///
/// The list is otherwise only fetched at login, and an empty cache leaves the tunnel blocking all
/// traffic — which blocks the fetch too, so a failed login-time fetch needs a way back.
struct RelayReloadButton: View {
    @Environment(DependencyManager.self) private var dependencies: DependencyManager?
    @State private var isReloading = false
    @State private var relayCount: Int?
    @State private var lastError: Error?

    var body: some View {
        VStack(spacing: 8) {
            Button(isReloading ? "Reloading relays…" : "Reload relays") {
                Task { await reload() }
            }
            .disabled(isReloading || dependencies == nil)

            if let lastError {
                Text(lastError.localizedDescription).font(.caption).foregroundStyle(Color.red)
            } else if let relayCount {
                Text(relayCount == 0 ? "No relays cached" : "\(relayCount) relays cached")
                    .font(.caption)
                    .foregroundStyle(relayCount == 0 ? Color.red : Color.secondary)
            }
        }
        .task { relayCount = dependencies?.makeRelayListUpdater().cachedRelayCount }
    }

    private func reload() async {
        guard let updater = dependencies?.makeRelayListUpdater() else { return }
        isReloading = true
        lastError = nil
        do {
            try await updater.updateRelays()
        } catch {
            lastError = error
        }
        relayCount = updater.cachedRelayCount
        isReloading = false
    }
}

// FIXME: Use MockProxyFactory.makeProxyFactory here instead ?
#Preview("Root View") {
    RootView(viewModel: LoginViewModelStub()).environment(Navigation())
}
