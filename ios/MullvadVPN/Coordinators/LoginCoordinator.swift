//
//  LoginCoordinator.swift
//  MullvadVPN
//
//  Created by pronebird on 27/01/2023.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Combine
import MullvadREST
import MullvadTypes
import Operations
import Routing
import SwiftUI
import UIKit

final class LoginCoordinator: Coordinator, Presenting {
    private let tunnelManager: TunnelManager
    private let devicesProxy: DeviceHandling

    private var loginController: LoginViewController?
    nonisolated(unsafe) private var lastLoginAction: LoginAction?
    private var subscriptions = Set<Combine.AnyCancellable>()

    var didFinish: (@MainActor @Sendable (LoginCoordinator) -> Void)?
    var didCreateAccount: (@MainActor @Sendable () -> Void)?

    var preferredAccountNumberPublisher: AnyPublisher<String, Never>?
    var presentationContext: UIViewController {
        navigationController
    }

    let navigationController: RootContainerViewController

    init(
        navigationController: RootContainerViewController,
        tunnelManager: TunnelManager,
        devicesProxy: DeviceHandling
    ) {
        self.navigationController = navigationController
        self.tunnelManager = tunnelManager
        self.devicesProxy = devicesProxy
    }

    func start(animated: Bool) {
        let interactor = LoginInteractor(tunnelManager: tunnelManager)
        let loginController = LoginViewController(
            interactor: interactor,
            alertPresenter: AlertPresenter(context: self)
        )

        loginController.didFinishLogin = { [weak self] action, error in
            self?.didFinishLogin(action: action, error: error) ?? .nothing
        }

        preferredAccountNumberPublisher?
            .compactMap { $0 }
            .sink(receiveValue: { preferredAccountNumber in
                interactor.suggestPreferredAccountNumber?(preferredAccountNumber)
            })
            .store(in: &subscriptions)

        interactor.didCreateAccount = didCreateAccount

        navigationController.pushViewController(loginController, animated: animated)

        self.loginController = loginController
    }

    // MARK: - Private

    private func didFinishLogin(action: LoginAction, error: Error?) -> EndLoginAction {
        guard let error else {
            callDidFinishAfterDelay()
            return .nothing
        }

        if case let .useExistingAccount(accountNumber) = action {
            if let error = error as? REST.Error, error.compareErrorCode(.maxDevicesReached) {
                return .wait(
                    Promise { resolve in
                        nonisolated(unsafe) let sendableResolve = resolve
                        self.showDeviceList(for: accountNumber) { error in
                            self.lastLoginAction = action

                            sendableResolve(error.map { .failure($0) } ?? .success(()))
                        }
                    })
            } else {
                return .activateTextField
            }
        }

        return .nothing
    }

    private func callDidFinishAfterDelay() {
        DispatchQueue.main.asyncAfter(deadline: .now() + .seconds(1)) { [weak self] in
            guard let self else { return }
            didFinish?(self)
        }
    }

    private func returnToLogin(repeatLogin: Bool) {
        navigationController.dismiss(animated: true) { [weak self] in
            if let lastLoginAction = self?.lastLoginAction, repeatLogin {
                self?.loginController?.start(action: lastLoginAction)
            }
        }
    }

    private func showDeviceList(for accountNumber: String, completion: @escaping @Sendable (Error?) -> Void) {
        let interactor = DeviceManagementInteractor(
            accountNumber: accountNumber,
            devicesProxy: devicesProxy
        )
        let controller = UIHostingController(
            rootView: DeviceManagementView(
                deviceManaging: interactor,
                style: .tooManyDevices(returnToLogin),
                onError: { title, error in
                    let errorDescription =
                        if case let .network(urlError) = error as? REST.Error {
                            urlError.localizedDescription
                        } else {
                            error.localizedDescription
                        }
                    let presentation = AlertPresentation(
                        id: "delete-device-error-alert",
                        title: title,
                        message: errorDescription,
                        buttons: [
                            AlertAction(
                                title: NSLocalizedString("Got it!", comment: ""),
                                style: .default
                            )
                        ]
                    )

                    let presenter = AlertPresenter(context: self)
                    presenter.showAlert(presentation: presentation, animated: true)
                }
            )
        )
        controller.navigationItem.rightBarButtonItem = UIBarButtonItem(
            systemItem: .cancel,
            primaryAction: UIAction(handler: { _ in
                controller.dismiss(animated: true)
            })
        )
        controller.isModalInPresentation = true
        navigationController
            .present(
                CustomNavigationController(rootViewController: controller),
                animated: true
            ) {
                completion(nil)
            }
    }
}
