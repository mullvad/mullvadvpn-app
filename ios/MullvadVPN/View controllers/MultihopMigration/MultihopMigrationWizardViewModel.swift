//
//  MultihopMigrationWizardViewModel.swift
//  MullvadVPN
//
//  Created by Mojgan on 2026-04-21.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Combine
import Foundation
import MullvadSettings
import SwiftUI

protocol MultihopMigrationWizardViewModelProtocol: ObservableObject {
    var items: [StateViewModel] { get }
}

final class MultihopMigrationWizardViewModel: MultihopMigrationWizardViewModelProtocol {
    let output: MigrationOutput<MultihopStateV2>
    var items: [StateViewModel] = []
    var tunnelManager: TunnelManager
    var tunnelObserver: TunnelBlockObserver

    init(tunnelManager: TunnelManager, output: MigrationOutput<MultihopStateV2>) {
        self.output = output
        self.tunnelManager = tunnelManager
        self.items = output.changes.compactMap {
            let descriptor = SettingsUpdateDescriptor(change: $0)
            return StateViewModel(
                style: .info,
                title: TextItem(text: descriptor.title, style: .headline),
                banner: descriptor.banner,
                details: descriptor.description)
        }
        let tunnelObserver = TunnelBlockObserver(didUpdateTunnelStatus: { tunnelManager, tunnelStatus in

        })
        self.tunnelObserver = tunnelObserver
    }
}
