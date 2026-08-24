// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import Combine
import MullvadSettings
import MullvadTypes
import Routing
import UIKit

class AddAccessMethodCoordinator: Coordinator, Presentable, Presenting {
    private let subject: CurrentValueSubject<AccessMethodViewModel, Never> = .init(AccessMethodViewModel())

    let navigationController: UINavigationController
    let accessMethodRepository: AccessMethodRepositoryProtocol
    let proxyConfigurationTester: ProxyConfigurationTesterProtocol
    let interactor: EditAccessMethodInteractor

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

        interactor = EditAccessMethodInteractor(
            subject: subject,
            repository: accessMethodRepository,
            proxyConfigurationTester: proxyConfigurationTester
        )
    }

    func start() {
        let controller = MethodSettingsViewController(
            subject: subject,
            interactor: interactor,
            alertPresenter: AlertPresenter(context: self)
        )

        setUpControllerNavigationItem(controller)
        controller.delegate = self

        LocalNetworkProbe().triggerLocalNetworkPrivacyAlert()
        navigationController.pushViewController(controller, animated: false)
    }

    private func setUpControllerNavigationItem(_ controller: MethodSettingsViewController) {
        controller.navigationItem.prompt = NSLocalizedString("The app will test the method before saving.", comment: "")

        controller.navigationItem.title = NSLocalizedString("Add method", comment: "")

        controller.saveBarButton.title = NSLocalizedString("Add", comment: "")

        controller.navigationItem.leftBarButtonItem = UIBarButtonItem(
            systemItem: .cancel,
            primaryAction: UIAction(handler: { [weak self] _ in
                self?.dismiss(animated: true)
            })
        )
    }
}

extension AddAccessMethodCoordinator: @preconcurrency MethodSettingsViewControllerDelegate {
    func accessMethodDidSave(_ accessMethod: PersistentAccessMethod) {
        dismiss(animated: true)
    }

    func controllerShouldShowProtocolPicker(_ controller: MethodSettingsViewController) {
        let picker = AccessMethodProtocolPicker(navigationController: navigationController)

        picker.present(currentValue: subject.value.method) { [weak self] newMethod in
            self?.subject.value.method = newMethod
        }
    }

    func controllerShouldShowShadowsocksCipherPicker(_ controller: MethodSettingsViewController) {
        let picker = ShadowsocksCipherPicker(
            navigationController: navigationController, ciphers: interactor.shadowsocksCiphers)

        picker.present(currentValue: interactor.shadowsocksCiphers.first ?? "") { [weak self] selectedCipher in
            self?.subject.value.shadowsocks.cipher = selectedCipher
        }
    }
}
