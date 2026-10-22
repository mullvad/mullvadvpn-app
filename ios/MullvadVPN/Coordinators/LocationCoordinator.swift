// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import MullvadREST
import MullvadSettings
import MullvadTypes
import Routing
import SwiftUI

class LocationCoordinator: Coordinator, Presentable, Presenting {
    private let tunnelManager: TunnelManager
    private var tunnelObserver: TunnelObserver?
    private let relaySelectorWrapper: RelaySelectorWrapper
    private let relayCacheTracker: RelayCacheTrackerProtocol
    private let customListRepository: CustomListRepositoryProtocol
    private let recentConnectionsRepository: RecentConnectionsRepositoryProtocol

    let navigationController: UINavigationController

    var presentedViewController: UIViewController {
        navigationController
    }

    var selectLocationViewModel: (any SelectLocationViewModel)!

    var didFinish: ((LocationCoordinator) -> Void)?

    init(
        navigationController: UINavigationController,
        tunnelManager: TunnelManager,
        relaySelectorWrapper: RelaySelectorWrapper,
        relayCacheTracker: RelayCacheTrackerProtocol,
        customListRepository: CustomListRepositoryProtocol,
        recentConnectionsRepository: RecentConnectionsRepositoryProtocol
    ) {
        self.navigationController = navigationController
        self.tunnelManager = tunnelManager
        self.relaySelectorWrapper = relaySelectorWrapper
        self.relayCacheTracker = relayCacheTracker
        self.customListRepository = customListRepository
        self.recentConnectionsRepository = recentConnectionsRepository
    }

    func start() {
        let selectLocationViewModelImpl = SelectLocationViewModelImpl(
            tunnelManager: tunnelManager,
            relaySelectorWrapper: relaySelectorWrapper,
            relayCacheTracker: relayCacheTracker,
            customListRepository: customListRepository,
            recentConnectionsRepository: recentConnectionsRepository,
            delegate: .init(
                showDaitaSettings: { [weak self] in
                    self?.navigateToDaitaSettings()
                },
                showObfuscationSettings: { [weak self] in
                    self?.navigateToObfuscationSettings()
                },
                showIpVersionSettings: { [weak self] in
                    self?.navigateToIpVersionSettings()
                },
                showFilterView: { [weak self] multihopContext in
                    self?.navigateToFilter(multihopContext: multihopContext)
                },
                didSelectExitRelayLocations: { [weak self] constraint in
                    guard let self else { return }
                    self.didSelectExitRelays(constraint)
                    self.didFinish?(self)
                },
                didSelectEntryRelayLocations: { [weak self] constraint in
                    self?.didSelectEntryRelays(constraint)
                },
                didFinish: { [weak self] in
                    guard let self else { return }
                    self.didFinish?(self)
                }
            )
        )
        selectLocationViewModel = selectLocationViewModelImpl
        let hostingController = UIHostingController(
            rootView: SelectLocationView(
                viewModel: selectLocationViewModelImpl)
        )
        hostingController.view.setAccessibilityIdentifier(.selectLocationView)

        navigationController.pushViewController(hostingController, animated: false)
    }
}

extension LocationCoordinator {
    func navigateToFilter(multihopContext: MultihopContext) {
        let relayFilterCoordinator = RelayFilterCoordinator(
            navigationController: CustomNavigationController(),
            tunnelManager: tunnelManager,
            multihopContext: multihopContext,
            relaySelectorWrapper: relaySelectorWrapper
        )

        relayFilterCoordinator.didFinish = {
            relayFilterCoordinator.dismiss(animated: true)
        }
        relayFilterCoordinator.onFeatureChipTapped = { [weak self] feature in
            switch feature {
            case .daita: self?.navigateToDaitaSettings()
            case .obfuscation: self?.navigateToObfuscationSettings()
            default: break
            }
        }
        relayFilterCoordinator.start()

        presentChild(relayFilterCoordinator, animated: true)
    }

    func didSelectEntryRelays(_ constraint: RelayConstraint<UserSelectedRelays>) {
        var relayConstraints = tunnelManager.settings.relayConstraints
        relayConstraints.entryLocations = constraint

        tunnelManager.updateSettings([.relayConstraints(relayConstraints)]) {
            self.tunnelManager.startTunnel()
        }
    }

    func navigateToDaitaSettings() {
        applicationRouter?.present(.daita)
    }

    func navigateToObfuscationSettings() {
        applicationRouter?.present(.vpnSettings(.obfuscation))
    }

    func navigateToIpVersionSettings() {
        applicationRouter?.present(.vpnSettings(.ipVersion))
    }

    func didSelectExitRelays(_ constraint: RelayConstraint<UserSelectedRelays>) {
        var relayConstraints = tunnelManager.settings.relayConstraints
        relayConstraints.exitLocations = constraint

        tunnelManager.updateSettings([.relayConstraints(relayConstraints)]) {
            self.tunnelManager.startTunnel()
        }
    }
}
