//
//  DebugCoordinator.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2026-02-27.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Routing
import SwiftUI

class DebugCoordinator: Coordinator, Presentable, Presenting {
    private let navigationController: UINavigationController
    private let viewModel: DebugViewModelImpl
    //    private let alertPresenter: AlertPresenter

    var presentedViewController: UIViewController {
        navigationController
    }

    var didFinish: ((DebugCoordinator) -> Void)?

    init(
        navigationController: UINavigationController,
        viewModel: DebugViewModelImpl
    ) {
        self.navigationController = navigationController
        self.viewModel = viewModel

        super.init()

        //        alertPresenter = AlertPresenter(context: self)
    }

    func start(animated: Bool) {
        let view = DebugView(viewModel: viewModel)

        let host = UIHostingController(rootView: view)
        host.title = NSLocalizedString("Debug", comment: "")
        host.view.setAccessibilityIdentifier(.debugView)
        host.view.backgroundColor = .secondaryColor

        navigationController.pushViewController(host, animated: animated)
    }
}
