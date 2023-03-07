//
//  OutOfTimeCoordinator.swift
//  MullvadVPN
//
//  Created by pronebird on 10/03/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import UIKit

class OutOfTimeCoordinator: Coordinator, OutOfTimeViewControllerDelegate {
    let navigationController: RootContainerViewController
    let storePaymentManager: StorePaymentManager
    let tunnelManager: TunnelManager

    var didFinishPayment: ((OutOfTimeCoordinator) -> Void)?

    private(set) var isMakingPayment = false
    private var viewController: OutOfTimeViewController?

    init(
        navigationController: RootContainerViewController,
        storePaymentManager: StorePaymentManager,
        tunnelManager: TunnelManager
    ) {
        self.navigationController = navigationController
        self.storePaymentManager = storePaymentManager
        self.tunnelManager = tunnelManager
    }

    func start(animated: Bool) {
        let interactor = OutOfTimeInteractor(
            storePaymentManager: storePaymentManager,
            tunnelManager: tunnelManager
        )
        let controller = OutOfTimeViewController(interactor: interactor)
        controller.delegate = self

        viewController = controller

        navigationController.pushViewController(controller, animated: animated)
    }

    func popFromNavigationStack(animated: Bool, completion: @escaping () -> Void) {
        guard let viewController = viewController else {
            completion()
            return
        }

        let viewControllers = navigationController.viewControllers.filter { $0 != viewController }

        navigationController.setViewControllers(
            viewControllers,
            animated: animated,
            completion: completion
        )
    }

    // MARK: - OutOfTimeViewControllerDelegate

    func outOfTimeViewControllerDidBeginPayment(_ controller: OutOfTimeViewController) {
        isMakingPayment = true
    }

    func outOfTimeViewControllerDidEndPayment(_ controller: OutOfTimeViewController) {
        isMakingPayment = false

        didFinishPayment?(self)
    }
}
