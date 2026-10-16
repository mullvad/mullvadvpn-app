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
import MullvadTypes
import Routing
import UIKit

class RelayFilterCoordinator: Coordinator, Presentable {
    private let tunnelManager: TunnelManager
    private let relaySelectorWrapper: RelaySelectorWrapper
    private let multihopContext: MultihopContext

    let navigationController: UINavigationController

    var presentedViewController: UIViewController {
        return navigationController
    }

    var didFinish: (() -> Void)?
    var onFeatureChipTapped: ((SelectLocationFilter) -> Void)?

    init(
        navigationController: UINavigationController,
        tunnelManager: TunnelManager,
        multihopContext: MultihopContext,
        relaySelectorWrapper: RelaySelectorWrapper
    ) {
        self.navigationController = navigationController
        self.tunnelManager = tunnelManager
        self.multihopContext = multihopContext
        self.relaySelectorWrapper = relaySelectorWrapper
    }

    func start() {
        let viewModel = RelayFilterSelection.ViewModel(
            tunnelManager: tunnelManager,
            relaySelectorWrapper: relaySelectorWrapper,
            multihopContext: multihopContext
        )
        let relayFilterView = RelayFilterView(viewModel: viewModel)

        viewModel.onApplyFilter = { [weak self] filter in
            guard let self else { return }

            var relayConstraints = tunnelManager.settings.relayConstraints
            relayConstraints.setFilterConstraint(.only(filter), for: multihopContext)
            tunnelManager.updateSettings([.relayConstraints(relayConstraints)])

            didFinish?()
        }

        viewModel.onCancel = { [weak self] in
            self?.didFinish?()
        }

        viewModel.onFeatureChipTapped = { [weak self] feature in
            self?.onFeatureChipTapped?(feature)
        }

        let host = UIHostingRootController(rootView: relayFilterView)
        navigationController.pushViewController(host, animated: false)
    }
}
