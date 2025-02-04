//
//  IPOverrideCoordinator.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-01-15.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import MullvadSettings
import MullvadTypes
import Routing
import UIKit

class IPOverrideCoordinator: Coordinator, Presenting, SettingsChildCoordinator {
    private let navigationController: UINavigationController
    private let interactor: IPOverrideInteractor

    var presentationContext: UIViewController {
        navigationController
    }

    init(
        navigationController: UINavigationController,
        repository: IPOverrideRepositoryProtocol,
        tunnelManager: TunnelManager
    ) {
        self.navigationController = navigationController
        interactor = IPOverrideInteractor(repository: repository, tunnelManager: tunnelManager)
    }

    func start(animated: Bool) {
        let controller = IPOverrideViewController(
            interactor: interactor,
            alertPresenter: AlertPresenter(context: self)
        )

        controller.delegate = self

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
        let header = NSLocalizedString(
            "IP_OVERRIDE_HEADER",
            tableName: "IPOverride",
            value: "IP Override",
            comment: ""
        )
        let body = [
            NSLocalizedString(
                "IP_OVERRIDE_BODY_1",
                tableName: "IPOverride",
                value: """
                On some networks, where various types of censorship are being used, our server IP addresses are \
                sometimes blocked.
                """,
                comment: ""
            ),
            NSLocalizedString(
                "IP_OVERRIDE_BODY_2",
                tableName: "IPOverride",
                value: """
                To circumvent this you can import a file or a text, provided by our support team, \
                with new IP addresses that override the default addresses of the servers in the Select location view.
                """,
                comment: ""
            ),
            NSLocalizedString(
                "IP_OVERRIDE_BODY_3",
                tableName: "IPOverride",
                value: """
                If you are having issues connecting to VPN servers, please contact support.
                """,
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
