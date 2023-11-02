//
//  AddAccessMethodCoordinator.swift
//  MullvadVPN
//
//  Created by pronebird on 08/11/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Combine
import Routing
import UIKit

class AddAccessMethodCoordinator: Coordinator, Presentable {
    private let subject: CurrentValueSubject<AccessMethodViewModel, Never> = .init(AccessMethodViewModel())

    var presentedViewController: UIViewController {
        navigationController
    }

    let navigationController: UINavigationController
    let accessMethodRepo: AccessMethodRepositoryProtocol
    let proxyConfigurationTester: ProxyConfigurationTesterProtocol

    init(
        navigationController: UINavigationController,
        accessMethodRepo: AccessMethodRepositoryProtocol,
        proxyConfigurationTester: ProxyConfigurationTesterProtocol
    ) {
        self.navigationController = navigationController
        self.accessMethodRepo = accessMethodRepo
        self.proxyConfigurationTester = proxyConfigurationTester
    }

    func start() {
        let controller = AddAccessMethodViewController(
            subject: subject,
            interactor: AddAccessMethodInteractor(
                subject: subject,
                repo: accessMethodRepo,
                proxyConfigurationTester: proxyConfigurationTester
            )
        )
        controller.delegate = self

        navigationController.pushViewController(controller, animated: false)
    }
}

extension AddAccessMethodCoordinator: AddAccessMethodViewControllerDelegate {
    func controllerDidAdd(_ controller: AddAccessMethodViewController) {
        dismiss(animated: true)
    }

    func controllerDidCancel(_ controller: AddAccessMethodViewController) {
        dismiss(animated: true)
    }

    func controllerShouldShowProtocolPicker(_ controller: AddAccessMethodViewController) {
        let picker = AccessMethodProtocolPicker(navigationController: navigationController)

        picker.present(currentValue: subject.value.method) { [weak self] newMethod in
            self?.subject.value.method = newMethod
        }
    }

    func controllerShouldShowShadowsocksCipherPicker(_ controller: AddAccessMethodViewController) {
        let picker = ShadowsocksCipherPicker(navigationController: navigationController)

        picker.present(currentValue: subject.value.shadowsocks.cipher) { [weak self] selectedCipher in
            self?.subject.value.shadowsocks.cipher = selectedCipher
        }
    }
}
