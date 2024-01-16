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
        // swiftlint:disable line_length
        let aboutMarkdown = """
        **What is Lorem Ipsum?**
        Lorem Ipsum is simply dummy text of the printing and typesetting industry. Lorem Ipsum has been the industry's standard dummy text ever since the 1500s, when an unknown printer took a galley of type and scrambled it to make a type specimen book. It has survived not only five centuries, but also the leap into electronic typesetting, remaining essentially unchanged. It was popularised in the 1960s with the release of Letraset sheets containing Lorem Ipsum passages, and more recently with desktop publishing software like Aldus PageMaker including versions of Lorem Ipsum.
        """
        // swiftlint:enable line_length

        let aboutController = AboutViewController(markdown: aboutMarkdown)
        let aboutNavController = UINavigationController(rootViewController: aboutController)

        aboutController.navigationItem.title = NSLocalizedString(
            "ABOUT_API_ACCESS_NAV_TITLE",
            value: "About API access",
            comment: ""
        )

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
