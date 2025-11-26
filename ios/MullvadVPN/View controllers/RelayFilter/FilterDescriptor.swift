//
//  FilterDescriptor.swift
//  MullvadVPN
//
//  Created by Mojgan on 2025-02-25.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//
import MullvadREST
import MullvadSettings
import MullvadTypes

struct FilterDescriptor {
    let relayFilterResult: RelayCandidates
    let settings: LatestTunnelSettings
    let multihopContext: MultihopContext

    var isEnabled: Bool {
        numberOfServers >= 1
    }

    var title: String {
        guard isEnabled else {
            return NSLocalizedString("No matching servers found.", comment: "")
        }
        return createTitleForAvailableServers()
    }

    var description: String {
        return if shouldShowDaitaDescription {
            NSLocalizedString("When using DAITA, one provider with DAITA-enabled servers is required.", comment: "")
        } else {
            ""
        }
    }

    var shouldShowDaitaDescription: Bool {
        let isDaitaEnabled = settings.daita.daitaState.isEnabled
        let isAutomaticRoutingEnabled = settings.daita.isAutomaticRouting
        let isMultihopEnabled = settings.tunnelMultihopState.isEnabled

        return switch multihopContext {
        case .entry:
            isDaitaEnabled
        case .exit:
            isDaitaEnabled && !isAutomaticRoutingEnabled && !isMultihopEnabled
        }
    }

    init(relayFilterResult: RelayCandidates, settings: LatestTunnelSettings, multihopContext: MultihopContext) {
        self.relayFilterResult = relayFilterResult
        self.settings = settings
        self.multihopContext = multihopContext
    }

    private var numberOfServers: Int {
        switch multihopContext {
        case .entry:
            (relayFilterResult.entryRelays ?? []).count
        case .exit:
            relayFilterResult.exitRelays.count
        }
    }

    private func createTitleForAvailableServers() -> String {
        let displayNumber: (Int) -> String = { number in
            number >= 100 ? "99+" : "\(number)"
        }
        return String(format: NSLocalizedString("Show %@ servers", comment: ""), displayNumber(numberOfServers))
    }
}
