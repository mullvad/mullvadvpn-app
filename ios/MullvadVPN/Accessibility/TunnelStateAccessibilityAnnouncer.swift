//
//  TunnelStateAccessibilityAnnouncer.swift
//  MullvadVPN
//
//  Created on 2026-02-16.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Foundation
import UIKit

/// Posts VoiceOver announcements when the tunnel state changes so that
/// blind/low-vision users are always informed, regardless of focus.
@MainActor
final class TunnelStateAccessibilityAnnouncer {
    private var tunnelObserver: TunnelBlockObserver?
    private var lastAnnouncement: String?
    private var pendingAnnouncementTask: Task<Void, Never>?

    init(tunnelManager: TunnelManager) {
        let tunnelObserver = TunnelBlockObserver(
            didUpdateTunnelStatus: { [weak self] _, tunnelStatus in
                let state = tunnelStatus.state
                Task { @MainActor [weak self] in
                    self?.handleTunnelStateChange(state)
                }
            }
        )
        self.tunnelObserver = tunnelObserver
        tunnelManager.addObserver(tunnelObserver)

        // Announce current tunnel state on startup after VoiceOver finishes
        // reading the initial screen elements.
        let currentState = tunnelManager.tunnelStatus.state
        pendingAnnouncementTask = Task { [weak self] in
            try? await Task.sleep(for: .seconds(4))
            guard !Task.isCancelled else { return }
            guard UIAccessibility.isVoiceOverRunning else { return }
            guard let self, let announcement = announcementString(for: currentState) else { return }
            lastAnnouncement = announcement
            UIAccessibility.post(notification: .announcement, argument: announcement)
        }
    }

    private func handleTunnelStateChange(_ state: TunnelState) {
        guard UIAccessibility.isVoiceOverRunning else { return }
        guard let announcement = announcementString(for: state) else { return }

        // Avoid repeating the same announcement.
        guard announcement != lastAnnouncement else { return }
        lastAnnouncement = announcement

        // Cancel any pending announcement to avoid stale reads.
        pendingAnnouncementTask?.cancel()

        // Delay so VoiceOver finishes reading any changed button labels
        // (e.g., Connect → Cancel → Disconnect) before the announcement.
        pendingAnnouncementTask = Task { [weak self] in
            try? await Task.sleep(for: .seconds(1.5))
            guard !Task.isCancelled else { return }
            if self == nil { return }
            UIAccessibility.post(notification: .announcement, argument: announcement)
        }
    }

    // MARK: - State to announcement mapping

    /// Returns a localized announcement string, or `nil` for transient states that should be silent.
    private nonisolated func announcementString(for state: TunnelState) -> String? {
        switch state {
        case let .connecting(relays, _, _):
            if let relays {
                String(
                    format: NSLocalizedString(
                        "TUNNEL_STATE_CONNECTING_TO",
                        value: "Connecting to %@, %@",
                        comment: "VoiceOver announcement when connecting to a relay"
                    ),
                    relays.exit.location.city,
                    relays.exit.location.country
                )
            } else {
                NSLocalizedString(
                    "TUNNEL_STATE_CONNECTING",
                    value: "Connecting",
                    comment: "VoiceOver announcement when connecting without relay info"
                )
            }

        case let .connected(relays, _, _):
            String(
                format: NSLocalizedString(
                    "TUNNEL_STATE_CONNECTED_TO",
                    value: "Connected to %@, %@",
                    comment: "VoiceOver announcement when connected to a relay"
                ),
                relays.exit.location.city,
                relays.exit.location.country
            )

        case .disconnecting(.nothing):
            NSLocalizedString(
                "TUNNEL_STATE_DISCONNECTING",
                value: "Disconnecting",
                comment: "VoiceOver announcement when disconnecting"
            )

        case .disconnected:
            NSLocalizedString(
                "TUNNEL_STATE_DISCONNECTED",
                value: "Disconnected",
                comment: "VoiceOver announcement when disconnected"
            )

        case let .reconnecting(relays, _, _):
            String(
                format: NSLocalizedString(
                    "TUNNEL_STATE_RECONNECTING_TO",
                    value: "Reconnecting to %@, %@",
                    comment: "VoiceOver announcement when reconnecting to a relay"
                ),
                relays.exit.location.city,
                relays.exit.location.country
            )

        case .waitingForConnectivity(.noConnection):
            NSLocalizedString(
                "TUNNEL_STATE_BLOCKED_CONNECTION",
                value: "Blocked connection",
                comment: "VoiceOver announcement when connection is blocked"
            )

        case .waitingForConnectivity(.noNetwork):
            NSLocalizedString(
                "TUNNEL_STATE_NO_NETWORK",
                value: "No network",
                comment: "VoiceOver announcement when no network is available"
            )

        case let .error(reason):
            String(
                format: NSLocalizedString(
                    "TUNNEL_STATE_BLOCKED_WITH_REASON",
                    value: "Blocked connection: %@",
                    comment: "VoiceOver announcement for blocked/error state with reason"
                ),
                reason.localizedReason
            )

        // Transient/intermediate states — stay silent.
        case .pendingReconnect,
            .negotiatingEphemeralPeer,
            .disconnecting(.reconnect):
            nil
        }
    }
}
