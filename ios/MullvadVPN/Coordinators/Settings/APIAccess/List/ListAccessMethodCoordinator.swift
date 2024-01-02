//
//  ListAccessMethodCoordinator.swift
//  MullvadVPN
//
//  Created by pronebird on 08/11/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import MullvadSettings
import Routing
import UIKit

class ListAccessMethodCoordinator: Coordinator, Presenting, SettingsChildCoordinator {
    let navigationController: UINavigationController
    let accessMethodRepository: AccessMethodRepositoryProtocol
    let proxyConfigurationTester: ProxyConfigurationTester = .shared

    var presentationContext: UIViewController {
        navigationController
    }

    init(
        navigationController: UINavigationController,
        accessMethodRepository: AccessMethodRepositoryProtocol
    ) {
        self.navigationController = navigationController
        self.accessMethodRepository = accessMethodRepository
    }

    func start(animated: Bool) {
        let listController = ListAccessMethodViewController(
            interactor: ListAccessMethodInteractor(repo: accessMethodRepository)
        )
        listController.delegate = self
        navigationController.pushViewController(listController, animated: animated)
    }

    private func addNew() {
        let coordinator = AddAccessMethodCoordinator(
            navigationController: CustomNavigationController(),
            accessMethodRepo: accessMethodRepository,
            proxyConfigurationTester: proxyConfigurationTester
        )

        coordinator.start()
        presentChild(coordinator, animated: true)
    }

    private func edit(item: ListAccessMethodItem) {
        // Remove previous edit coordinator to prevent accumulation.
        childCoordinators.filter { $0 is EditAccessMethodCoordinator }.forEach { $0.removeFromParent() }

        let editCoordinator = EditAccessMethodCoordinator(
            navigationController: navigationController,
            accessMethodRepo: accessMethodRepository,
            proxyConfigurationTester: proxyConfigurationTester,
            methodIdentifier: item.id
        )
        editCoordinator.onFinish = { [weak self] coordinator in
            self?.popToList()
            coordinator.removeFromParent()
        }
        editCoordinator.start()
        addChild(editCoordinator)
    }

    private func popToList() {
        guard let listController = navigationController.viewControllers
            .first(where: { $0 is ListAccessMethodViewController }) else { return }

        navigationController.popToViewController(listController, animated: true)
    }

    private func about() {
        let header = NSLocalizedString(
            "ABOUT_API_ACCESS_HEADER",
            value: "API access",
            comment: ""
        )
        let preamble = NSLocalizedString(
            "ABOUT_API_ACCESS_PREAMBLE",
            value: "Manage default and setup custom methods to access the Mullvad API.",
            comment: ""
        )
        let body = [
            NSLocalizedString(
                "ABOUT_API_ACCESS_BODY_1",
                value: """
                The app needs to communicate with a Mullvad API server to log you in, fetch server lists, \
                and other critical operations.
                """,
                comment: ""
            ),
            NSLocalizedString(
                "ABOUT_API_ACCESS_BODY_2",
                value: """
                On some networks, where various types of censorship are being used, the API servers might \
                not be directly reachable.
                """,
                comment: ""
            ),
            NSLocalizedString(
                "ABOUT_API_ACCESS_BODY_3",
                value: """
                This feature allows you to circumvent that censorship by adding custom ways to access the \
                API via proxies and similar methods.
                """,
                comment: ""
            ),
        ]

        let aboutController = AboutViewController(header: header, preamble: preamble, body: body)
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

extension ListAccessMethodCoordinator: ListAccessMethodViewControllerDelegate {
    func controllerShouldShowAbout(_ controller: ListAccessMethodViewController) {
        about()
    }

    func controllerShouldAddNew(_ controller: ListAccessMethodViewController) {
        addNew()
    }

    func controller(_ controller: ListAccessMethodViewController, shouldEditItem item: ListAccessMethodItem) {
        edit(item: item)
    }
}
