//
//  EditAccessMethodCoordinator.swift
//  MullvadVPN
//
//  Created by pronebird on 21/11/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Combine
import MullvadSettings
import Routing
import UIKit

class EditAccessMethodCoordinator: Coordinator {
    let navigationController: UINavigationController
    let subject: CurrentValueSubject<AccessMethodViewModel, Never> = .init(AccessMethodViewModel())
    let accessMethodRepo: AccessMethodRepositoryProtocol
    let proxyConfigurationTester: ProxyConfigurationTester
    let methodIdentifier: UUID

    var onFinish: ((EditAccessMethodCoordinator) -> Void)?

    init(
        navigationController: UINavigationController,
        accessMethodRepo: AccessMethodRepositoryProtocol,
        proxyConfigurationTester: ProxyConfigurationTester,
        methodIdentifier: UUID
    ) {
        self.navigationController = navigationController
        self.accessMethodRepo = accessMethodRepo
        self.proxyConfigurationTester = proxyConfigurationTester
        self.methodIdentifier = methodIdentifier
    }

    func start() {
        guard let persistentMethod = accessMethodRepo.fetch(by: methodIdentifier) else { return }

        subject.value = persistentMethod.toViewModel()

        let interactor = EditAccessMethodInteractor(
            subject: subject,
            repo: accessMethodRepo,
            proxyConfigurationTester: proxyConfigurationTester
        )
        let controller = EditAccessMethodViewController(subject: subject, interactor: interactor)
        controller.delegate = self

        navigationController.pushViewController(controller, animated: true)
    }
}

extension EditAccessMethodCoordinator: EditAccessMethodViewControllerDelegate {
    func controllerDidSaveAccessMethod(_ controller: EditAccessMethodViewController) {
        onFinish?(self)
    }

    func controllerShouldShowProxyConfiguration(_ controller: EditAccessMethodViewController) {
        let interactor = EditAccessMethodInteractor(
            subject: subject,
            repo: accessMethodRepo,
            proxyConfigurationTester: proxyConfigurationTester
        )
        let controller = ProxyConfigurationViewController(subject: subject, interactor: interactor)
        controller.delegate = self

        navigationController.pushViewController(controller, animated: true)
    }

    func controllerDidDeleteAccessMethod(_ controller: EditAccessMethodViewController) {
        onFinish?(self)
    }
}

extension EditAccessMethodCoordinator: ProxyConfigurationViewControllerDelegate {
    func controllerShouldShowProtocolPicker(_ controller: ProxyConfigurationViewController) {
        let picker = AccessMethodProtocolPicker(navigationController: navigationController)

        picker.present(currentValue: subject.value.method) { [weak self] newMethod in
            self?.subject.value.method = newMethod
        }
    }

    func controllerShouldShowShadowsocksCipherPicker(_ controller: ProxyConfigurationViewController) {
        let picker = ShadowsocksCipherPicker(navigationController: navigationController)

        picker.present(currentValue: subject.value.shadowsocks.cipher) { [weak self] selectedCipher in
            self?.subject.value.shadowsocks.cipher = selectedCipher
        }
    }
}
