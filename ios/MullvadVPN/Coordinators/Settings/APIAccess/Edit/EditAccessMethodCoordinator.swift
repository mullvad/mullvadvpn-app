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

class EditAccessMethodCoordinator: Coordinator, Presenting {
    let navigationController: UINavigationController
    let accessMethodRepository: AccessMethodRepositoryProtocol
    let proxyConfigurationTester: ProxyConfigurationTester
    let methodIdentifier: UUID
    var methodSettingsSubject: CurrentValueSubject<AccessMethodViewModel, Never> = .init(AccessMethodViewModel())

    var onFinish: ((EditAccessMethodCoordinator) -> Void)?

    var presentationContext: UIViewController {
        navigationController
    }

    init(
        navigationController: UINavigationController,
        accessMethodRepo: AccessMethodRepositoryProtocol,
        proxyConfigurationTester: ProxyConfigurationTester,
        methodIdentifier: UUID
    ) {
        self.navigationController = navigationController
        self.accessMethodRepository = accessMethodRepo
        self.proxyConfigurationTester = proxyConfigurationTester
        self.methodIdentifier = methodIdentifier
    }

    func start() {
        let subject = getViewModelSubjectFromStore()

        let interactor = EditAccessMethodInteractor(
            subject: subject,
            repository: accessMethodRepository,
            proxyConfigurationTester: proxyConfigurationTester
        )

        let controller = EditAccessMethodViewController(
            subject: subject,
            interactor: interactor,
            alertPresenter: AlertPresenter(context: self)
        )
        controller.delegate = self

        navigationController.pushViewController(controller, animated: true)
    }
}

extension EditAccessMethodCoordinator: EditAccessMethodViewControllerDelegate {
    func controllerShouldShowMethodSettings(_ controller: EditAccessMethodViewController) {
        methodSettingsSubject = getViewModelSubjectFromStore()

        let interactor = EditAccessMethodInteractor(
            subject: methodSettingsSubject,
            repository: accessMethodRepository,
            proxyConfigurationTester: proxyConfigurationTester
        )

        let controller = MethodSettingsViewController(
            subject: methodSettingsSubject,
            interactor: interactor,
            alertPresenter: AlertPresenter(context: self)
        )

        controller.navigationItem.prompt = NSLocalizedString(
            "METHOD_SETTINGS_NAVIGATION_ADD_PROMPT",
            tableName: "APIAccess",
            value: "The app will test the method before saving.",
            comment: ""
        )

        controller.navigationItem.title = NSLocalizedString(
            "METHOD_SETTINGS_NAVIGATION_ADD_TITLE",
            tableName: "APIAccess",
            value: "Method settings",
            comment: ""
        )

        controller.saveBarButton.title = NSLocalizedString(
            "METHOD_SETTINGS_NAVIGATION_ADD_BUTTON",
            tableName: "APIAccess",
            value: "Save",
            comment: ""
        )

        controller.delegate = self

        navigationController.pushViewController(controller, animated: true)
    }

    func controllerDidDeleteAccessMethod(_ controller: EditAccessMethodViewController) {
        onFinish?(self)
    }

    private func getViewModelSubjectFromStore() -> CurrentValueSubject<AccessMethodViewModel, Never> {
        let persistentMethod = accessMethodRepository.fetch(by: methodIdentifier)
        return CurrentValueSubject<AccessMethodViewModel, Never>(persistentMethod?.toViewModel() ?? .init())
    }
}

extension EditAccessMethodCoordinator: MethodSettingsViewControllerDelegate {
    func controllerDidSaveAccessMethod(_ controller: MethodSettingsViewController) {
        navigationController.popViewController(animated: true)
    }

    func controllerShouldShowProtocolPicker(_ controller: MethodSettingsViewController) {
        let picker = AccessMethodProtocolPicker(navigationController: navigationController)

        picker.present(currentValue: methodSettingsSubject.value.method) { [weak self] newMethod in
            self?.methodSettingsSubject.value.method = newMethod
        }
    }

    func controllerShouldShowShadowsocksCipherPicker(_ controller: MethodSettingsViewController) {
        let picker = ShadowsocksCipherPicker(navigationController: navigationController)

        picker.present(currentValue: methodSettingsSubject.value.shadowsocks.cipher) { [weak self] selectedCipher in
            self?.methodSettingsSubject.value.shadowsocks.cipher = selectedCipher
        }
    }
}
