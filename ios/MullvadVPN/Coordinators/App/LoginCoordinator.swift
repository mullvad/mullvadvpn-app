//
//  LoginCoordinator.swift
//  MullvadVPN
//
//  Created by pronebird on 27/01/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import MullvadREST
import MullvadTypes
import Operations
import UIKit

final class LoginCoordinator: Coordinator, DeviceManagementViewControllerDelegate {
    private let tunnelManager: TunnelManager
    private let devicesProxy: REST.DevicesProxy

    private var loginController: LoginViewController?
    private var lastLoginAction: LoginAction?

    var didFinish: ((LoginCoordinator) -> Void)?

    let navigationController: RootContainerViewController

    init(
        navigationController: RootContainerViewController,
        tunnelManager: TunnelManager,
        devicesProxy: REST.DevicesProxy
    ) {
        self.navigationController = navigationController
        self.tunnelManager = tunnelManager
        self.devicesProxy = devicesProxy
    }

    func start(animated: Bool) {
        let interactor = LoginInteractor(tunnelManager: tunnelManager)
        let loginController = LoginViewController(interactor: interactor)

        loginController.didFinishLogin = { [weak self] action, error in
            return self?.didFinishLogin(action: action, error: error) ?? .nothing
        }

        navigationController.pushViewController(loginController, animated: animated)

        self.loginController = loginController
    }

    // MARK: - DeviceManagementViewControllerDelegate

    func deviceManagementViewControllerDidCancel(_ controller: DeviceManagementViewController) {
        returnToLogin(repeatLogin: false)
    }

    func deviceManagementViewControllerDidFinish(_ controller: DeviceManagementViewController) {
        returnToLogin(repeatLogin: true)
    }

    // MARK: - Private

    private func didFinishLogin(action: LoginAction, error: Error?) -> EndLoginAction {
        guard let error = error else {
            callDidFinishAfterDelay()
            return .nothing
        }

        if case let .useExistingAccount(accountNumber) = action {
            if let error = error as? REST.Error, error.compareErrorCode(.maxDevicesReached) {
                return .wait(Promise { resolve in
                    self.showDeviceList(for: accountNumber) { error in
                        self.lastLoginAction = action

                        resolve(error.map { .failure($0) } ?? .success(()))
                    }
                })
            } else {
                return .activateTextField
            }
        }

        return .nothing
    }

    private func callDidFinishAfterDelay() {
        DispatchQueue.main.asyncAfter(wallDeadline: .now() + .seconds(1)) { [weak self] in
            guard let self = self else { return }
            self.didFinish?(self)
        }
    }

    private func returnToLogin(repeatLogin: Bool) {
        guard let loginController = loginController else { return }

        navigationController.popToViewController(loginController, animated: true) {
            if let lastLoginAction = self.lastLoginAction, repeatLogin {
                self.loginController?.start(action: lastLoginAction)
            }
        }
    }

    private func showDeviceList(for accountNumber: String, completion: @escaping (Error?) -> Void) {
        let interactor = DeviceManagementInteractor(
            accountNumber: accountNumber,
            devicesProxy: devicesProxy
        )
        let controller = DeviceManagementViewController(interactor: interactor)
        controller.delegate = self
        controller.fetchDevices(animateUpdates: false) { [weak self] result in
            guard let self = self else { return }

            switch result {
            case .success:
                self.navigationController.pushViewController(controller, animated: true) {
                    completion(nil)
                }

            case let .failure(error):
                completion(error)
            }
        }
    }
}
