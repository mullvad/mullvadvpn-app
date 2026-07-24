//
//  SettingsMigrationWizardViewModel.swift
//  MullvadVPN
//
//  Created by Mojgan on 2026-04-21.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Combine
import Foundation
import MullvadSettings
import SwiftUI

protocol SettingsMigrationWizardViewModelProtocol: ObservableObject {
    var items: [StateViewModel] { get }
}

final class SettingsMigrationWizardViewModel: SettingsMigrationWizardViewModelProtocol {
    var items: [StateViewModel] = []

    private var tunnelManager: TunnelManager
    private var settings: LatestTunnelSettings
    private var actionItem: MullvadStateView.ActionItem

    private var actionDescriptor: MultihopActionDescriptor?
    private var tunnelObserver: TunnelBlockObserver?

    private var isVpnConnectionActive: Bool {
        switch tunnelManager.tunnelStatus.state {
        case .connected, .connecting, .reconnecting, .negotiatingEphemeralPeer:
            return true
        default:
            return false
        }
    }

    init(
        tunnelManager: TunnelManager,
        output: MigrationResult<MultihopStateV2, MultihopSuggestedAction>
    ) {
        self.tunnelManager = tunnelManager
        self.settings = tunnelManager.settings
        self.actionItem = MullvadStateView.ActionItem(style: .primary, state: .init(kind: .idle, message: ""))

        let changeItems = output.changes.map { change in
            let descriptor = SettingsUpdateDescriptor(
                change: change
            )

            return StateViewModel(
                style: .info,
                title: MullvadStateView.TextItem(
                    text: descriptor.title,
                    style: .headline()
                ),
                banner: descriptor.banner,
                details: descriptor.description
            )
        }

        let actionItems: [StateViewModel] =
            output.action.map { suggestedAction in
                let descriptor = MultihopActionDescriptor(action: suggestedAction)
                self.actionDescriptor = descriptor

                let oldSettings = settings
                suggestedAction.action?(&settings)
                self.actionItem.state =
                    oldSettings == settings ? descriptor.makeState(for: .success) : descriptor.makeState(for: .idle)

                actionItem.onTap = {
                    [weak self] in
                    guard let self else { return }
                    tunnelManager.updateSettings([
                        .multihop(settings.tunnelMultihopState),
                        .relayConstraints(settings.relayConstraints),
                    ])
                    guard isVpnConnectionActive else {
                        actionItem.state = descriptor.makeState(for: .success)
                        return
                    }
                    actionItem.state = descriptor.makeState(for: .loading)
                }

                return [
                    StateViewModel(
                        style: .info,
                        title: MullvadStateView.TextItem(
                            text: descriptor.title,
                            style: .headline()
                        ),
                        banner: descriptor.banner,
                        details: descriptor.description,
                        actions: [actionItem]
                    )
                ]
            } ?? []

        self.items = changeItems + actionItems

        let tunnelObserver = TunnelBlockObserver(
            didUpdateTunnelStatus: { [weak self] _, tunnelStatus in
                guard let self, let actionDescriptor = self.actionDescriptor else { return }
                if case .connected = tunnelStatus.state {
                    actionItem.state = actionDescriptor.makeState(for: .success)
                }
            }
        )

        self.tunnelObserver = tunnelObserver

        tunnelManager.addObserver(tunnelObserver)
    }
}
