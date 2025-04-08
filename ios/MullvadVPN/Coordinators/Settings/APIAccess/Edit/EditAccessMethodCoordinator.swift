//
//  EditAccessMethodCoordinator.swift
//  MullvadVPN
//
//  Created by pronebird on 21/11/2023.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Combine
import MullvadSettings
import MullvadTypes
import Routing
import UIKit

class EditAccessMethodCoordinator: Coordinator, Presenting {
    let navigationController: UINavigationController
    let proxyConfigurationTester: ProxyConfigurationTesterProtocol
    let accessMethodRepository: AccessMethodRepositoryProtocol
    let methodIdentifier: UUID
    var methodSettingsSubject: CurrentValueSubject<AccessMethodViewModel, Never> = .init(AccessMethodViewModel())
    var editAccessMethodSubject: CurrentValueSubject<AccessMethodViewModel, Never> = .init(AccessMethodViewModel())

    var onFinish: ((EditAccessMethodCoordinator) -> Void)?

    var presentationContext: UIViewController {
        navigationController
    }

    init(
        navigationController: UINavigationController,
        accessMethodRepo: AccessMethodRepositoryProtocol,
        proxyConfigurationTester: ProxyConfigurationTesterProtocol,
        methodIdentifier: UUID
    ) {
        self.navigationController = navigationController
        self.accessMethodRepository = accessMethodRepo
        self.proxyConfigurationTester = proxyConfigurationTester
        self.methodIdentifier = methodIdentifier
    }

    func start() {
        editAccessMethodSubject = getViewModelSubjectFromStore()

        let interactor = EditAccessMethodInteractor(
            subject: editAccessMethodSubject,
            repository: accessMethodRepository,
            proxyConfigurationTester: proxyConfigurationTester
        )

        let controller = EditAccessMethodViewController(
            subject: editAccessMethodSubject,
            interactor: interactor,
            alertPresenter: AlertPresenter(context: self)
        )
        controller.delegate = self

        navigationController.pushViewController(controller, animated: true)
    }
}

extension EditAccessMethodCoordinator: @preconcurrency EditAccessMethodViewControllerDelegate {
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

    func controllerShouldShowMethodInfo(_ controller: EditAccessMethodViewController, config: InfoModalConfig) {
        let aboutController = AboutViewController(
            header: config.header,
            preamble: config.preamble,
            body: config.body
        )
        let aboutNavController = UINavigationController(rootViewController: aboutController)

        aboutController.navigationItem.rightBarButtonItem = UIBarButtonItem(
            systemItem: .done,
            primaryAction: UIAction { [weak aboutNavController] _ in
                aboutNavController?.dismiss(animated: true)
            }
        )

        navigationController.present(aboutNavController, animated: true)
    }

    private func getViewModelSubjectFromStore() -> CurrentValueSubject<AccessMethodViewModel, Never> {
        let persistentMethod = accessMethodRepository.fetch(by: methodIdentifier)
        return CurrentValueSubject<AccessMethodViewModel, Never>(persistentMethod?.toViewModel() ?? .init())
    }
}

extension EditAccessMethodCoordinator: @preconcurrency MethodSettingsViewControllerDelegate {
    func accessMethodDidSave(_ accessMethod: PersistentAccessMethod) {
        editAccessMethodSubject.value = accessMethod.toViewModel()
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
