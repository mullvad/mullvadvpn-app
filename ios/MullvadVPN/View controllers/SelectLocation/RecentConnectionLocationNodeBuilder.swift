//
//  RecentConnectionLocationNodeBuilder.swift
//  MullvadVPN
//
//  Created by Mojgan on 2025-09-30.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//
import Foundation
import MullvadSettings
import MullvadTypes

struct RecentConnectionLocationNodeBuilder {
    let settings: LatestTunnelSettings
    let recentConnection: RecentConnection
    let userSelectedLocationFinder: UserSelectedLocationFinder

    init(
        userSelectedLocationFinder: UserSelectedLocationFinder,
        settings: LatestTunnelSettings,
        recentConnection: RecentConnection
    ) {
        self.settings = settings
        self.recentConnection = recentConnection
        self.userSelectedLocationFinder = userSelectedLocationFinder
    }

    var recentConnectionLocationNode: RecentConnectionLocationNode? {
        let entrySelectedNode: LocationNode? = if let entryNode = recentConnection.entry {
            userSelectedLocationFinder.node(entryNode)
        } else {
            nil
        }
        
        guard let exitSelectedNode = userSelectedLocationFinder.node(recentConnection.exit) else {
            return nil
        }
        
        if !settings.tunnelMultihopState.isEnabled {
            return RecentConnectionLocationNode(
                name: exitSelectedNode.name,
                code: exitSelectedNode.code,
                isActive: true,
                entryLocation: nil,
                exitLocation: exitSelectedNode
            )
        } else if let entrySelectedNode{
            return RecentConnectionLocationNode(
                name: "\(exitSelectedNode.name) via \(entrySelectedNode.name)",
                code: "",
                isActive: true,
                entryLocation: entrySelectedNode,
                exitLocation: exitSelectedNode
            )
        }
        return nil
    }
}
