//
//  AddAccessMethodCoordinator.swift
//  MullvadVPN
//
//  Created by pronebird on 08/11/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Combine
import MullvadSettings
import Routing
import UIKit

class AddAccessMethodCoordinator: Coordinator, Presentable, Presenting {
    private let subject: CurrentValueSubject<AccessMethodViewModel, Never> = .init(AccessMethodViewModel())

    let navigationController: UINavigationController
    let accessMethodRepository: AccessMethodRepositoryProtocol
    let proxyConfigurationTester: ProxyConfigurationTesterProtocol

    var presentedViewController: UIViewController {
        navigationController
    }

    init(
        navigationController: UINavigationController,
        accessMethodRepo: AccessMethodRepositoryProtocol,
        proxyConfigurationTester: ProxyConfigurationTesterProtocol
    ) {
        self.navigationController = navigationController
        self.accessMethodRepository = accessMethodRepo
        self.proxyConfigurationTester = proxyConfigurationTester
    }

    func start() {
        let controller = MethodSettingsViewController(
            subject: subject,
            interactor: EditAccessMethodInteractor(
                subject: subject,
                repository: accessMethodRepository,
                proxyConfigurationTester: proxyConfigurationTester
            ),
            alertPresenter: AlertPresenter(context: self)
        )

        setUpControllerNavigationItem(controller)
        controller.delegate = self

        navigationController.pushViewController(controller, animated: false)
    }

    private func setUpControllerNavigationItem(_ controller: MethodSettingsViewController) {
        controller.navigationItem.prompt = NSLocalizedString(
            "METHOD_SETTINGS_NAVIGATION_ADD_PROMPT",
            tableName: "APIAccess",
            value: "The app will test the method before saving.",
            comment: ""
        )

        controller.navigationItem.title = NSLocalizedString(
            "METHOD_SETTINGS_NAVIGATION_ADD_TITLE",
            tableName: "APIAccess",
            value: "Add access method",
            comment: ""
        )

        controller.saveBarButton.title = NSLocalizedString(
            "METHOD_SETTINGS_NAVIGATION_ADD_BUTTON",
            tableName: "APIAccess",
            value: "Add",
            comment: ""
        )

        controller.navigationItem.leftBarButtonItem = UIBarButtonItem(
            systemItem: .cancel,
            primaryAction: UIAction(handler: { [weak self] _ in
                self?.dismiss(animated: true)
            })
        )
    }
}

extension AddAccessMethodCoordinator: MethodSettingsViewControllerDelegate {
    func viewModelDidSave(_ viewModel: AccessMethodViewModel) {
        dismiss(animated: true)
    }

    func controllerShouldShowProtocolPicker(_ controller: MethodSettingsViewController) {
        let picker = AccessMethodProtocolPicker(navigationController: navigationController)

        picker.present(currentValue: subject.value.method) { [weak self] newMethod in
            self?.subject.value.method = newMethod
        }
    }

    func controllerShouldShowShadowsocksCipherPicker(_ controller: MethodSettingsViewController) {
        let picker = ShadowsocksCipherPicker(navigationController: navigationController)

        picker.present(currentValue: subject.value.shadowsocks.cipher) { [weak self] selectedCipher in
            self?.subject.value.shadowsocks.cipher = selectedCipher
        }
    }
}
