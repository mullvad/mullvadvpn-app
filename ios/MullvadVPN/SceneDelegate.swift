//
//  SceneDelegate.swift
//  MullvadVPN
//
//  Created by pronebird on 20/05/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import MullvadLogging
import MullvadREST
import MullvadTypes
import Operations
import RelayCache
import UIKit

class SceneDelegate: UIResponder, UIWindowSceneDelegate, UISplitViewControllerDelegate,
    RootContainerViewControllerDelegate, LoginViewControllerDelegate,
    DeviceManagementViewControllerDelegate, SettingsNavigationControllerDelegate,
    OutOfTimeViewControllerDelegate, SelectLocationViewControllerDelegate,
    RevokedDeviceViewControllerDelegate, NotificationManagerDelegate, TunnelObserver,
    RelayCacheTrackerObserver, SettingsMigrationUIHandler
{
    private let logger = Logger(label: "SceneDelegate")

    var window: UIWindow?
    private var privacyOverlayWindow: UIWindow?
    private var isSceneConfigured = false

    private let rootContainer = RootContainerViewController()

    // Modal root container is used on iPad to present login, TOS, revoked device, device management
    // view controllers above `rootContainer` which only contains split controller.
    private lazy var modalRootContainer = RootContainerViewController()
    private lazy var modalRootAdaptivePresentationDelegate = ModalRootAdaptivePresentationDelegate(
        parentRootContainer: rootContainer,
        modalRootContainer: modalRootContainer
    )

    private var splitViewController: CustomSplitViewController?
    private var selectLocationViewController: SelectLocationViewController?
    private var tunnelViewController: TunnelViewController?
    private weak var settingsNavController: SettingsNavigationController?
    private var lastLoginAction: LoginAction?

    private var accountDataThrottling: AccountDataThrottling?
    private var deviceDataThrottling: DeviceDataThrottling?

    private var outOfTimeTimer: Timer?

    private var appDelegate: AppDelegate {
        return UIApplication.shared.delegate as! AppDelegate
    }

    private var storePaymentManager: StorePaymentManager {
        return appDelegate.storePaymentManager
    }

    private var relayCacheTracker: RelayCacheTracker {
        return appDelegate.relayCacheTracker
    }

    private var tunnelManager: TunnelManager {
        return appDelegate.tunnelManager
    }

    private var apiProxy: REST.APIProxy {
        return appDelegate.apiProxy
    }

    private var devicesProxy: REST.DevicesProxy {
        return appDelegate.devicesProxy
    }

    deinit {
        clearOutOfTimeTimer()
    }

    var isShowingOutOfTimeView: Bool {
        switch UIDevice.current.userInterfaceIdiom {
        case .pad:
            return modalRootContainer.viewControllers
                .contains(where: { $0 is OutOfTimeViewController })
        case .phone:
            return rootContainer.viewControllers
                .contains(where: { $0 is OutOfTimeViewController })
        default:
            return false
        }
    }

    func showUserAccount() {
        rootContainer.showSettings(navigateTo: .account, animated: true)
    }

    private func configureScene() {
        guard !isSceneConfigured else { return }

        isSceneConfigured = true

        accountDataThrottling = AccountDataThrottling(tunnelManager: tunnelManager)
        deviceDataThrottling = DeviceDataThrottling(tunnelManager: tunnelManager)

        rootContainer.delegate = self
        window?.rootViewController = rootContainer

        switch UIDevice.current.userInterfaceIdiom {
        case .pad:
            setupPadUI()
        case .phone:
            setupPhoneUI()
        default:
            fatalError()
        }

        relayCacheTracker.addObserver(self)
        NotificationManager.shared.delegate = self

        refreshDeviceAndAccountData(forceUpdate: true)
    }

    private func setShowsPrivacyOverlay(_ showOverlay: Bool) {
        if showOverlay {
            privacyOverlayWindow?.isHidden = false
            privacyOverlayWindow?.makeKeyAndVisible()
        } else {
            privacyOverlayWindow?.isHidden = true
            window?.makeKeyAndVisible()
        }
    }

    private func refreshDeviceAndAccountData(forceUpdate: Bool) {
        let condition: AccountDataThrottling.Condition =
            settingsNavController == nil && !forceUpdate
                ? .whenCloseToExpiryAndBeyond
                : .always

        accountDataThrottling?.requestUpdate(condition: condition)
        deviceDataThrottling?.requestUpdate(forceUpdate: forceUpdate)
    }

    private func resetDeviceAndAccountDataThrottling() {
        accountDataThrottling?.reset()
        deviceDataThrottling?.reset()
    }

    private func showSelectLocationController() {
        let contentController = makeSelectLocationController()
        contentController.navigationItem.rightBarButtonItem = UIBarButtonItem(
            barButtonSystemItem: .done,
            target: self,
            action: #selector(handleDismissSelectLocationController(_:))
        )

        let navController = SelectLocationNavigationController(contentController: contentController)
        rootContainer.present(navController, animated: true)

        selectLocationViewController = contentController
    }

    @objc private func handleDismissSelectLocationController(_ sender: Any) {
        selectLocationViewController?.dismiss(animated: true)
    }

    // MARK: - UIWindowSceneDelegate

    func scene(
        _ scene: UIScene,
        willConnectTo session: UISceneSession,
        options connectionOptions: UIScene.ConnectionOptions
    ) {
        guard let windowScene = scene as? UIWindowScene else { return }

        window = UIWindow(windowScene: windowScene)
        window?.rootViewController = LaunchViewController()

        privacyOverlayWindow = UIWindow(windowScene: windowScene)
        privacyOverlayWindow?.rootViewController = LaunchViewController()
        privacyOverlayWindow?.windowLevel = .alert + 1

        window?.makeKeyAndVisible()

        tunnelManager.addObserver(self)
        if tunnelManager.isConfigurationLoaded {
            configureScene()
        }
    }

    func sceneDidDisconnect(_ scene: UIScene) {}

    func sceneDidBecomeActive(_ scene: UIScene) {
        if isSceneConfigured {
            refreshDeviceAndAccountData(forceUpdate: false)
        }

        setShowsPrivacyOverlay(false)
    }

    func sceneWillResignActive(_ scene: UIScene) {
        setShowsPrivacyOverlay(true)
    }

    func sceneWillEnterForeground(_ scene: UIScene) {}

    func sceneDidEnterBackground(_ scene: UIScene) {}

    // MARK: - OutOfTimeViewControllerDelegate

    func outOfTimeViewControllerDidBeginPayment(_ controller: OutOfTimeViewController) {
        setEnableSettingsButton(isEnabled: false, from: controller)
    }

    func outOfTimeViewControllerDidEndPayment(_ controller: OutOfTimeViewController) {
        setEnableSettingsButton(isEnabled: true, from: controller)
    }

    // MARK: - RootContainerViewControllerDelegate

    func rootContainerViewControllerShouldShowSettings(
        _ controller: RootContainerViewController,
        navigateTo route: SettingsNavigationRoute?,
        animated: Bool
    ) {
        // Check if settings controller is already presented.
        if let settingsNavController = settingsNavController {
            settingsNavController.navigate(to: route ?? .root, animated: animated)
        } else {
            let navController = makeSettingsNavigationController(route: route)

            // Refresh account data each time user opens settings
            refreshDeviceAndAccountData(forceUpdate: true)

            // On iPad the login controller can be presented modally above the root container.
            // in that case we have to use the presented controller to present the next modal.
            if let presentedController = controller.presentedViewController {
                presentedController.present(navController, animated: true)
            } else {
                controller.present(navController, animated: true)
            }

            // Save the reference for later.
            settingsNavController = navController
        }
    }

    func rootContainerViewSupportedInterfaceOrientations(_ controller: RootContainerViewController)
        -> UIInterfaceOrientationMask
    {
        switch UIDevice.current.userInterfaceIdiom {
        case .pad:
            return [.landscape, .portrait]
        case .phone:
            return [.portrait]
        default:
            return controller.supportedInterfaceOrientations
        }
    }

    func rootContainerViewAccessibilityPerformMagicTap(_ controller: RootContainerViewController)
        -> Bool
    {
        guard tunnelManager.deviceState.isLoggedIn else { return false }

        switch tunnelManager.tunnelStatus.state {
        case .connected, .connecting, .reconnecting, .waitingForConnectivity:
            tunnelManager.reconnectTunnel(selectNewRelay: true)

        case .disconnecting, .disconnected:
            tunnelManager.startTunnel()

        case .pendingReconnect:
            break
        }

        return true
    }

    private func setupPadUI() {
        let selectLocationController = makeSelectLocationController()
        let tunnelController = makeTunnelViewController()

        let splitViewController = CustomSplitViewController()
        splitViewController.delegate = self
        splitViewController.minimumPrimaryColumnWidth = UIMetrics.minimumSplitViewSidebarWidth
        splitViewController.preferredPrimaryColumnWidthFraction = UIMetrics
            .maximumSplitViewSidebarWidthFraction
        splitViewController.primaryEdge = .trailing
        splitViewController.dividerColor = UIColor.MainSplitView.dividerColor
        splitViewController.viewControllers = [selectLocationController, tunnelController]

        selectLocationViewController = selectLocationController
        self.splitViewController = splitViewController
        tunnelViewController = tunnelController

        rootContainer.setViewControllers([splitViewController], animated: false)
        showSplitViewMaster(tunnelManager.deviceState.isLoggedIn, animated: false)

        modalRootContainer.delegate = self

        let showNextController = { [weak self] (animated: Bool) in
            guard let self = self else { return }

            lazy var viewControllers: [UIViewController] = [self.makeLoginController()]

            switch self.tunnelManager.deviceState {
            case .loggedIn:
                self.modalRootContainer.setViewControllers(
                    viewControllers,
                    animated: self.isModalRootPresented && animated
                )

                // Dismiss modal root container if needed before proceeding.
                self.dismissModalRootContainerIfNeeded(animated: animated) {
                    self.handleExpiredAccount()
                }
                return

            case .loggedOut:
                break

            case .revoked:
                viewControllers.append(self.makeRevokedDeviceController())
            }

            // Configure modal container.
            self.modalRootContainer.setViewControllers(
                viewControllers,
                animated: self.isModalRootPresented && animated
            )

            // Present modal container if not presented yet.
            self.presentModalRootContainerIfNeeded(animated: animated)
        }

        if TermsOfService.isAgreed {
            showNextController(false)
        } else {
            let termsOfServiceController = makeTermsOfServiceController { _ in
                showNextController(true)
            }

            modalRootContainer.setViewControllers([termsOfServiceController], animated: false)
            presentModalRootContainerIfNeeded(animated: false)
        }
    }

    private func presentModalRootContainerIfNeeded(animated: Bool) {
        guard !isModalRootPresented else { return }

        modalRootContainer.preferredContentSize = CGSize(width: 480, height: 600)
        modalRootContainer.presentationController?.delegate = modalRootAdaptivePresentationDelegate
        modalRootContainer.isModalInPresentation = true

        rootContainer.present(modalRootContainer, animated: animated)
    }

    private func dismissModalRootContainerIfNeeded(
        animated: Bool,
        completion: @escaping () -> Void
    ) {
        guard isModalRootPresented else {
            completion()
            return
        }

        modalRootContainer.dismiss(animated: animated, completion: completion)
    }

    private var isModalRootPresented: Bool {
        return modalRootContainer.presentingViewController != nil
    }

    private func setupPhoneUI() {
        let showNextController = { [weak self] (animated: Bool) in
            guard let self = self else { return }

            var viewControllers: [UIViewController] = [self.makeLoginController()]

            switch self.tunnelManager.deviceState {
            case .loggedIn:
                let tunnelViewController = self.makeTunnelViewController()
                self.tunnelViewController = tunnelViewController
                viewControllers.append(tunnelViewController)

            case .loggedOut:
                break

            case .revoked:
                viewControllers.append(self.makeRevokedDeviceController())
            }

            self.rootContainer.setViewControllers(viewControllers, animated: animated) {
                self.handleExpiredAccount()
            }
        }

        if TermsOfService.isAgreed {
            showNextController(false)
        } else {
            let termsOfServiceController = makeTermsOfServiceController { _ in
                showNextController(true)
            }
            rootContainer.setViewControllers([termsOfServiceController], animated: false)
        }
    }

    private func makeSettingsNavigationController(route: SettingsNavigationRoute?)
        -> SettingsNavigationController
    {
        let navController = SettingsNavigationController(
            interactorFactory: SettingsInteractorFactory(
                storePaymentManager: storePaymentManager,
                tunnelManager: tunnelManager,
                apiProxy: apiProxy
            )
        )
        navController.settingsDelegate = self

        if UIDevice.current.userInterfaceIdiom == .pad {
            navController.preferredContentSize = CGSize(width: 480, height: 568)
            navController.modalPresentationStyle = .formSheet
        }

        navController.presentationController?.delegate = navController

        if let route = route {
            navController.navigate(to: route, animated: false)
        }

        return navController
    }

    private func makeOutOfTimeViewController() -> OutOfTimeViewController {
        let viewController = OutOfTimeViewController(
            interactor: OutOfTimeInteractor(
                storePaymentManager: storePaymentManager,
                tunnelManager: tunnelManager
            )
        )
        viewController.delegate = self
        return viewController
    }

    private func makeTunnelViewController() -> TunnelViewController {
        let interactor = TunnelViewControllerInteractor(tunnelManager: tunnelManager)
        let tunnelViewController = TunnelViewController(interactor: interactor)
        tunnelViewController.shouldShowSelectLocationPicker = { [weak self] in
            self?.showSelectLocationController()
        }
        return tunnelViewController
    }

    private func makeSelectLocationController() -> SelectLocationViewController {
        let selectLocationController = SelectLocationViewController()
        selectLocationController.delegate = self

        if let cachedRelays = try? relayCacheTracker.getCachedRelays() {
            selectLocationController.setCachedRelays(cachedRelays)
        }

        let relayConstraints = tunnelManager.settings.relayConstraints

        selectLocationController.setSelectedRelayLocation(
            relayConstraints.location.value,
            animated: false,
            scrollPosition: .middle
        )

        return selectLocationController
    }

    private func makeTermsOfServiceController(
        completion: @escaping (UIViewController) -> Void
    ) -> TermsOfServiceViewController {
        let controller = TermsOfServiceViewController()

        if UIDevice.current.userInterfaceIdiom == .pad {
            controller.modalPresentationStyle = .formSheet
            controller.isModalInPresentation = true
        }

        controller.completionHandler = { controller in
            TermsOfService.setAgreed()
            completion(controller)
        }

        return controller
    }

    private func makeRevokedDeviceController() -> RevokedDeviceViewController {
        let controller = RevokedDeviceViewController(
            interactor: RevokedDeviceInteractor(tunnelManager: tunnelManager)
        )
        controller.delegate = self
        return controller
    }

    private func makeLoginController() -> LoginViewController {
        let controller = LoginViewController()
        controller.delegate = self
        return controller
    }

    private func handleExpiredAccount() {
        guard case let .loggedIn(accountData, _) = tunnelManager.deviceState,
              accountData.expiry <= Date() else { return }

        switch UIDevice.current.userInterfaceIdiom {
        case .phone:
            if !rootContainer.viewControllers.contains(where: { $0 is OutOfTimeViewController }) {
                rootContainer.pushViewController(makeOutOfTimeViewController(), animated: false)
            }
        case .pad:
            if !modalRootContainer.viewControllers
                .contains(where: { $0 is OutOfTimeViewController })
            {
                modalRootContainer.pushViewController(
                    makeOutOfTimeViewController(),
                    animated: false
                )
                presentModalRootContainerIfNeeded(animated: true)
            }
        default:
            return
        }
    }

    private func showSplitViewMaster(_ show: Bool, animated: Bool) {
        splitViewController?.preferredDisplayMode = show ? .allVisible : .primaryHidden
        tunnelViewController?.setMainContentHidden(!show, animated: animated)
    }

    private func showLoginViewAfterLogout(dismissController: UIViewController?) {
        switch UIDevice.current.userInterfaceIdiom {
        case .phone:
            let loginController = rootContainer.viewControllers.first as? LoginViewController
            loginController?.reset()

            rootContainer.popToRootViewController(animated: false)
            dismissController?.dismiss(animated: true)

        case .pad:
            let loginController = modalRootContainer.viewControllers.first as? LoginViewController
            loginController?.reset()

            let didDismissSourceController = {
                self.presentModalRootContainerIfNeeded(animated: true)
            }

            modalRootContainer.popToRootViewController(animated: false)
            showSplitViewMaster(false, animated: true)

            if let dismissController = dismissController {
                dismissController.dismiss(animated: true, completion: didDismissSourceController)
            } else {
                didDismissSourceController()
            }

        default:
            return
        }
    }

    private func dismissOutOfTimeController() {
        switch UIDevice.current.userInterfaceIdiom {
        case .phone:
            var viewControllers = rootContainer.viewControllers
            guard let outOfTimeControllerIndex = viewControllers
                .firstIndex(where: { $0 is OutOfTimeViewController }) else { return }
            viewControllers.remove(at: outOfTimeControllerIndex)
            rootContainer.setViewControllers(viewControllers, animated: true)
        case .pad:
            modalRootContainer.dismiss(animated: true)
        default:
            return
        }
    }

    private func showRevokedDeviceView() {
        switch UIDevice.current.userInterfaceIdiom {
        case .phone:
            guard let loginController = rootContainer.viewControllers.first as? LoginViewController
            else {
                return
            }

            loginController.reset()

            let viewControllers = [
                loginController,
                makeRevokedDeviceController(),
            ]

            rootContainer.setViewControllers(viewControllers, animated: true)

        case .pad:
            guard let loginController = modalRootContainer.viewControllers
                .first as? LoginViewController
            else {
                return
            }

            loginController.reset()

            let viewControllers = [
                loginController,
                makeRevokedDeviceController(),
            ]

            let didDismissSettings = {
                self.showSplitViewMaster(false, animated: true)
                self.presentModalRootContainerIfNeeded(animated: true)
            }

            modalRootContainer.setViewControllers(viewControllers, animated: isModalRootPresented)

            if let settingsNavController = settingsNavController {
                settingsNavController.dismiss(animated: true, completion: didDismissSettings)
            } else {
                didDismissSettings()
            }

        default:
            fatalError()
        }
    }

    // MARK: - LoginViewControllerDelegate

    func loginViewController(
        _ controller: LoginViewController,
        shouldHandleLoginAction action: LoginAction,
        completion: @escaping (OperationCompletion<StoredAccountData?, Error>) -> Void
    ) {
        setEnableSettingsButton(isEnabled: false, from: controller)

        tunnelManager.setAccount(action: action.setAccountAction) { operationCompletion in
            switch operationCompletion {
            case .success:
                // RootContainer's settings button will be re-enabled in
                // `loginViewControllerDidFinishLogin`
                completion(operationCompletion)

            case let .failure(error):
                // Show device management controller when too many devices detected during log in.
                if case let .useExistingAccount(accountNumber) = action,
                   let restError = error as? REST.Error,
                   restError.compareErrorCode(.maxDevicesReached)
                {
                    self.lastLoginAction = action

                    let deviceController = DeviceManagementViewController(
                        interactor: DeviceManagementInteractor(
                            accountNumber: accountNumber,
                            devicesProxy: self.devicesProxy
                        )
                    )
                    deviceController.delegate = self

                    deviceController
                        .fetchDevices(animateUpdates: false) { [weak self] operationCompletion in
                            controller.rootContainerController?.pushViewController(
                                deviceController,
                                animated: true
                            )

                            // Return .cancelled to login controller upon success.
                            completion(operationCompletion.flatMap { .cancelled })

                            self?.setEnableSettingsButton(isEnabled: true, from: controller)
                        }
                } else {
                    fallthrough
                }

            case .cancelled:
                self.setEnableSettingsButton(isEnabled: true, from: controller)
                completion(operationCompletion)
            }
        }
    }

    func loginViewControllerDidFinishLogin(_ controller: LoginViewController) {
        lastLoginAction = nil

        // Move the settings button back into header bar
        setEnableSettingsButton(isEnabled: true, from: controller)

        let relayConstraints = tunnelManager.settings.relayConstraints
        selectLocationViewController?.setSelectedRelayLocation(
            relayConstraints.location.value,
            animated: false,
            scrollPosition: .middle
        )

        switch UIDevice.current.userInterfaceIdiom {
        case .phone:
            let tunnelViewController = makeTunnelViewController()
            self.tunnelViewController = tunnelViewController
            var viewControllers = rootContainer.viewControllers
            viewControllers.append(tunnelViewController)
            rootContainer.setViewControllers(viewControllers, animated: true)
            handleExpiredAccount()

        case .pad:
            showSplitViewMaster(true, animated: true)

            dismissModalRootContainerIfNeeded(animated: true) {
                self.handleExpiredAccount()
            }

        default:
            fatalError()
        }
    }

    private func setUpOutOfTimeTimer() {
        outOfTimeTimer?.invalidate()

        guard case let .loggedIn(accountData, _) = tunnelManager.deviceState,
              accountData.expiry > Date() else { return }

        let timer = Timer(
            fire: accountData.expiry,
            interval: 0,
            repeats: false
        ) { [weak self] _ in
            self?.outOfTimeTimerDidFire()
        }

        outOfTimeTimer = timer
        RunLoop.main.add(timer, forMode: .common)
    }

    @objc func outOfTimeTimerDidFire() {
        handleExpiredAccount()
    }

    private func clearOutOfTimeTimer() {
        outOfTimeTimer?.invalidate()
        outOfTimeTimer = nil
    }

    private func setEnableSettingsButton(isEnabled: Bool, from viewController: UIViewController?) {
        let containers = [viewController?.rootContainerController, rootContainer].compactMap { $0 }

        for container in Set(containers) {
            container.setEnableSettingsButton(isEnabled)
        }
    }

    // MARK: - DeviceManagementViewControllerDelegate

    func deviceManagementViewControllerDidCancel(_ controller: DeviceManagementViewController) {
        controller.rootContainerController?.popViewController(animated: true)
    }

    func deviceManagementViewControllerDidFinish(_ controller: DeviceManagementViewController) {
        let currentRootContainer = controller.rootContainerController
        let loginViewController = currentRootContainer?.viewControllers
            .first as? LoginViewController

        currentRootContainer?.popViewController(animated: true) {
            if let lastLoginAction = self.lastLoginAction {
                loginViewController?.start(action: lastLoginAction)
            }
        }
    }

    // MARK: - SettingsNavigationControllerDelegate

    func settingsNavigationController(
        _ controller: SettingsNavigationController,
        willNavigateTo route: SettingsNavigationRoute
    ) {
        switch route {
        case .root, .account:
            refreshDeviceAndAccountData(forceUpdate: false)

        default:
            break
        }
    }

    func settingsNavigationController(
        _ controller: SettingsNavigationController,
        didFinishWithReason reason: SettingsDismissReason
    ) {
        if case .userLoggedOut = reason {
            showLoginViewAfterLogout(dismissController: controller)
        } else {
            controller.dismiss(animated: true)
        }
    }

    // MARK: - NotificationManagerDelegate

    func notificationManagerDidUpdateInAppNotifications(
        _ manager: NotificationManager,
        notifications: [InAppNotificationDescriptor]
    ) {
        tunnelViewController?.notificationController.setNotifications(notifications, animated: true)
    }

    // MARK: - SelectLocationViewControllerDelegate

    func selectLocationViewController(
        _ controller: SelectLocationViewController,
        didSelectRelayLocation relayLocation: RelayLocation
    ) {
        // Dismiss view controller in modal presentation
        if controller.presentingViewController != nil {
            window?.isUserInteractionEnabled = false
            DispatchQueue.main.asyncAfter(deadline: .now() + .milliseconds(250)) {
                self.window?.isUserInteractionEnabled = true
                controller.dismiss(animated: true) {
                    self.selectLocationControllerDidSelectRelayLocation(relayLocation)
                }
            }
        } else {
            selectLocationControllerDidSelectRelayLocation(relayLocation)
        }
    }

    private func selectLocationControllerDidSelectRelayLocation(_ relayLocation: RelayLocation) {
        let relayConstraints = RelayConstraints(location: .only(relayLocation))

        tunnelManager.setRelayConstraints(relayConstraints) {
            self.tunnelManager.startTunnel()
        }
    }

    // MARK: - RevokedDeviceViewControllerDelegate

    func revokedDeviceControllerDidRequestLogout(_ controller: RevokedDeviceViewController) {
        tunnelManager.unsetAccount { [weak self] in
            self?.showLoginViewAfterLogout(dismissController: nil)
        }
    }

    // MARK: - TunnelObserver

    func tunnelManagerDidLoadConfiguration(_ manager: TunnelManager) {
        configureScene()
    }

    func tunnelManager(_ manager: TunnelManager, didUpdateTunnelStatus tunnelStatus: TunnelStatus) {
        // no-op
    }

    func tunnelManager(
        _ manager: TunnelManager,
        didUpdateTunnelSettings tunnelSettings: TunnelSettingsV2
    ) {
        // no-op
    }

    func tunnelManager(_ manager: TunnelManager, didUpdateDeviceState deviceState: DeviceState) {
        switch deviceState {
        case let .loggedIn(accountData, _):
            if accountData.expiry > Date(),
               isShowingOutOfTimeView
            {
                dismissOutOfTimeController()
                setUpOutOfTimeTimer()
            }

        case .loggedOut:
            resetDeviceAndAccountDataThrottling()

        case .revoked:
            resetDeviceAndAccountDataThrottling()
            showRevokedDeviceView()
        }
    }

    func tunnelManager(_ manager: TunnelManager, didFailWithError error: Error) {
        // no-op
    }

    // MARK: - RelayCacheTrackerObserver

    func relayCacheTracker(
        _ tracker: RelayCacheTracker,
        didUpdateCachedRelays cachedRelays: CachedRelays
    ) {
        selectLocationViewController?.setCachedRelays(cachedRelays)
    }

    // MARK: - UISplitViewControllerDelegate

    func primaryViewController(forExpanding splitViewController: UISplitViewController)
        -> UIViewController?
    {
        // Restore the select location controller as primary when expanding the split view
        return selectLocationViewController
    }

    func primaryViewController(forCollapsing splitViewController: UISplitViewController)
        -> UIViewController?
    {
        // Set the connect controller as primary when collapsing the split view
        return tunnelViewController
    }

    func splitViewController(
        _ splitViewController: UISplitViewController,
        separateSecondaryFrom primaryViewController: UIViewController
    ) -> UIViewController? {
        // Dismiss the select location controller when expanding the split view
        if selectLocationViewController?.presentingViewController != nil {
            selectLocationViewController?.dismiss(animated: false)
        }
        return nil
    }

    // MARK: - SettingsMigrationUIHandler

    func showMigrationError(_ error: Error, completionHandler: @escaping () -> Void) {
        let alertController = UIAlertController(
            title: NSLocalizedString(
                "ALERT_TITLE",
                tableName: "SettingsMigrationUI",
                value: "Settings migration error",
                comment: ""
            ),
            message: Self.migrationErrorReason(error),
            preferredStyle: .alert
        )
        alertController.addAction(
            UIAlertAction(
                title: NSLocalizedString("OK", tableName: "SettingsMigrationUI", comment: ""),
                style: .default,
                handler: { _ in
                    completionHandler()
                }
            )
        )

        if let rootViewController = window?.rootViewController {
            rootViewController.present(alertController, animated: true)
        } else {
            completionHandler()
        }
    }

    private static func migrationErrorReason(_ error: Error) -> String {
        if error is UnsupportedSettingsVersionError {
            return NSLocalizedString(
                "NEWER_STORED_SETTINGS_ERROR",
                tableName: "SettingsMigrationUI",
                value: """
                The version of settings stored on device is from a newer app than is currently \
                running. Settings will be reset to defaults and device logged out.
                """,
                comment: ""
            )
        } else if let error = error as? SettingsMigrationError,
                  error.underlyingError is REST.Error
        {
            return NSLocalizedString(
                "NETWORK_ERROR",
                tableName: "SettingsMigrationUI",
                value: """
                Network error occurred. Settings will be reset to defaults and device logged out.
                """,
                comment: ""
            )
        } else {
            return NSLocalizedString(
                "INTERNAL_ERROR",
                tableName: "SettingsMigrationUI",
                value: """
                Internal error occurred. Settings will be reset to defaults and device logged out.
                """,
                comment: ""
            )
        }
    }
}
