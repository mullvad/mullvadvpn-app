//
//  Logging.swift
//  PacketTunnel
//
//  Created by pronebird on 18/12/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import Foundation
import os

private let kLogSubsystem = "net.mullvad.vpn.packet-tunnel"

/// A Wireguard event log
let wireguardLog = OSLog(subsystem: kLogSubsystem, category: "WireGuard")

/// A general tunnel provider log
let tunnelProviderLog = OSLog(subsystem: kLogSubsystem, category: "Tunnel Provider")

/// A WireguardDevice log
let wireguardDeviceLog = OSLog(subsystem: kLogSubsystem, category: "WireGuard Device")
