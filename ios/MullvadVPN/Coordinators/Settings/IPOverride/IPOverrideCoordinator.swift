//
//  IPOverrideCoordinator.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-01-15.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import MullvadSettings
import MullvadTypes
import Routing
import UIKit

class IPOverrideCoordinator: Coordinator, Presentable, Presenting, SettingsChildCoordinator {
    private let navigationController: UINavigationController
    private let interactor: IPOverrideInteractor
    private let route: AppRoute?
    var presentationContext: UIViewController {
        navigationController
    }

    var presentedViewController: UIViewController {
        navigationController
    }

    var didFinish: ((IPOverrideCoordinator) -> Void)?

    init(
        navigationController: UINavigationController,
        repository: IPOverrideRepositoryProtocol,
        tunnelManager: TunnelManager,
        route: AppRoute?
    ) {
        self.navigationController = navigationController
        interactor = IPOverrideInteractor(repository: repository, tunnelManager: tunnelManager)
        self.route = route
    }

    func start(animated: Bool) {
        let controller = IPOverrideViewController(
            interactor: interactor,
            alertPresenter: AlertPresenter(context: self)
        )

        controller.delegate = self

        if route == .ipOverrides {
            let doneButton = UIBarButtonItem(
                systemItem: .done,
                primaryAction: UIAction(handler: { [weak self] _ in
                    guard let self else { return }
                    didFinish?(self)
                })
            )
            controller.navigationItem.rightBarButtonItem = doneButton
        }

        navigationController.pushViewController(controller, animated: animated)
    }
}

extension IPOverrideCoordinator: @preconcurrency IPOverrideViewControllerDelegate {
    func presentImportTextController() {
        let viewController = IPOverrideTextViewController(interactor: interactor)
        let customNavigationController = CustomNavigationController(rootViewController: viewController)

        presentationContext.present(customNavigationController, animated: true)
    }

    func presentAbout() {
        let header = NSLocalizedString("Server IP override", comment: "")
        let body = [
            NSLocalizedString(
                """
                On some networks, where various types of censorship are being used, our server IP addresses are \
                sometimes blocked.
                """,
                comment: ""
            ),
            NSLocalizedString(
                "To circumvent this you can import a file or a text, provided by our support team, "
                    + "with new IP addresses that override the default addresses of the servers "
                    + "in the Select location view.",
                comment: ""
            ),
            NSLocalizedString(
                "If you are having issues connecting to VPN servers, please contact support.",
                comment: ""
            ),
        ]

        let aboutController = AboutViewController(header: header, preamble: nil, body: body)
        let aboutNavController = UINavigationController(rootViewController: aboutController)

        aboutController.navigationItem.rightBarButtonItem = UIBarButtonItem(
            systemItem: .done,
            primaryAction: UIAction { [weak aboutNavController] _ in
                aboutNavController?.dismiss(animated: true)
            }
        )

        navigationController.present(aboutNavController, animated: true)
    }
}
