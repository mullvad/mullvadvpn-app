//
//  StartOptions.swift
//  PacketTunnel
//
//  Created by pronebird on 03/08/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation

/// Packet tunnel start options parsed from dictionary passed to packet tunnel with a call to `startTunnel()`.
public struct StartOptions {
    /// The system that triggered the launch of packet tunnel.
    public var launchSource: LaunchSource

    /// Pre-selected relay received from UI when available.
    public var selectedRelay: SelectedRelay?

    /// Designated initializer.
    public init(launchSource: LaunchSource, selectedRelay: SelectedRelay? = nil) {
        self.launchSource = launchSource
        self.selectedRelay = selectedRelay
    }

    /// Returns a brief description suitable for output to tunnel provider log.
    public func logFormat() -> String {
        var s = "Start the tunnel via \(launchSource)"
        if let selectedRelay {
            s += ", connect to \(selectedRelay.hostname)"
        }
        s += "."
        return s
    }
}

/// The source facility that triggered a launch of packet tunnel extension.
public enum LaunchSource: String, CustomStringConvertible {
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
