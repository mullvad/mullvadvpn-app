// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import MullvadREST
import MullvadSettings
import MullvadTypes
import Routing
import SwiftUI

final class LoginCoordinator: Coordinator, Presenting {
    private let tunnelManager: TunnelManager
    private let devicesProxy: DeviceHandling
    private let breadcrumbsProvider: BreadcrumbsProvider
    private var breadcrumbsObserver: BreadcrumbsObserver?
    private var loginViewModel: LoginViewModel?

    var didFinish: (@MainActor @Sendable (LoginCoordinator) -> Void)?
    var didCreateAccount: (@MainActor @Sendable () -> Void)?
    var navigateToAccessMethods: (() -> Void)?

    var presentationContext: UIViewController {
        navigationController
    }

    let navigationController: RootContainerViewController
    let settingsManager: SettingsManager

    init(
        navigationController: RootContainerViewController,
        tunnelManager: TunnelManager,
        devicesProxy: DeviceHandling,
        breadcrumbsProvider: BreadcrumbsProvider,
        settingsManager: SettingsManager
    ) {
        self.navigationController = navigationController
        self.tunnelManager = tunnelManager
        self.devicesProxy = devicesProxy
        self.breadcrumbsProvider = breadcrumbsProvider
        self.settingsManager = settingsManager
    }

    func start(animated: Bool) {
        let interactor = LoginInteractor(tunnelManager: tunnelManager, settingsManager: settingsManager)
        interactor.didCreateAccount = didCreateAccount

        let viewModel = LoginViewModel(interactor: interactor)
        loginViewModel = viewModel

        viewModel.navigateToAccessMethods = navigateToAccessMethods
        viewModel.didFinishLogin = { [weak self] action, error in
            self?.didFinishLogin(action: action, error: error)
        }

        setUpBreadcrumbs()

        let controller = UIHostingRootController(rootView: LoginView(viewModel: viewModel))
        controller.view.setAccessibilityIdentifier(.loginView)

        navigationController.pushViewController(controller, animated: animated)
    }

    // MARK: - Private

    private func setUpBreadcrumbs() {
        loginViewModel?.showAccessMethodInvalidView = breadcrumbsProvider.breadcrumbs.contains(.warning(.apiAccess))

        let breadcrumbsObserver = BreadcrumbsBlockObserver(didUpdateBreadcrumbsHandler: { [weak self] in
            self?.loginViewModel?.showAccessMethodInvalidView = $0.contains(.warning(.apiAccess))
        })
        self.breadcrumbsObserver = breadcrumbsObserver
        breadcrumbsProvider.add(observer: breadcrumbsObserver)
    }

    private func didFinishLogin(action: LoginViewModel.Action, error: Error?) {
        guard let error else {
            callDidFinishAfterDelay()
            return
        }

        if case let .login(accountNumber) = action {
            if let error = error as? REST.Error, error.compareErrorCode(.maxDevicesReached) {
                self.showDeviceList(for: accountNumber)
            }
        }
    }

    private func callDidFinishAfterDelay() {
        DispatchQueue.main.asyncAfter(deadline: .now() + .seconds(1)) { [weak self] in
            guard let self else { return }
            didFinish?(self)
        }
    }

    private func returnToLogin(repeatLogin: Bool) {
        navigationController.dismiss(animated: true) { [weak self] in
            if repeatLogin {
                self?.loginViewModel?.login()
            }
        }
    }

    private func showDeviceList(for accountNumber: String) {
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
            )
    }
}
