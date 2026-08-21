// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import Foundation
import MullvadREST

/// Packet tunnel start options parsed from dictionary passed to packet tunnel with a call to `startTunnel()`.
public struct StartOptions: Sendable {
    /// The system that triggered the launch of packet tunnel.
    public var launchSource: LaunchSource

    /// Pre-selected relays received from UI when available.
    public var selectedRelays: SelectedRelays?

    /// Designated initializer.
    public init(launchSource: LaunchSource, selectedRelays: SelectedRelays? = nil) {
        self.launchSource = launchSource
        self.selectedRelays = selectedRelays
    }

    /// Returns a brief description suitable for output to tunnel provider log.
    public func logFormat() -> String {
        var s = "Start the tunnel via \(launchSource)"
        if let selectedRelays {
            s += ", connect to \(selectedRelays.exit.hostname)"
            s += selectedRelays.entry.flatMap { " via \($0.hostname)" } ?? ""
        }
        s += "."
        return s
    }
}

/// The source facility that triggered a launch of packet tunnel extension.
public enum LaunchSource: String, CustomStringConvertible, Sendable {
    /// Launched by the main bundle app using network extension framework.
    case app

    /// Launched via on-demand rule.
    case onDemand

    /// Launched by system, either on boot or via system VPN settings.
    case system

    /// Returns a human readable description of launch source.
    public var description: String {
        switch self {
        case .app, .system:
            return rawValue
        case .onDemand:
            return "on-demand rule"
        }
    }
}
