//
//  SceneDelegate.swift
//  MullvadVPN
//
//  Created by pronebird on 20/05/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Logging
import Operations
import UIKit

class SceneDelegate: UIResponder {
    private let logger = Logger(label: "SceneDelegate")

    var window: UIWindow?
    private var privacyOverlayWindow: UIWindow?
    private var isSceneConfigured = false

    private let rootContainer = RootContainerViewController()

    // Modal root container is used on iPad to present login, TOS, revoked device, device management
    // view controllers above `rootContainer` which only contains split controller.
    private lazy var modalRootContainer = RootContainerViewController()

    private var splitViewController: CustomSplitViewController?
    private var selectLocationViewController: SelectLocationViewController?
    private var connectController: ConnectViewController?
    private weak var settingsNavController: SettingsNavigationController?
    private var lastLoginAction: LoginAction?
    private var accountDataThrottling = AccountDataThrottling()
    private var outOfTimeTimer: Timer?

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

    func setupScene(windowFactory: WindowFactory) {
        window = windowFactory.create()
        window?.rootViewController = LaunchViewController()

        privacyOverlayWindow = windowFactory.create()
        privacyOverlayWindow?.rootViewController = LaunchViewController()
        privacyOverlayWindow?.windowLevel = .alert + 1

        window?.makeKeyAndVisible()

        TunnelManager.shared.addObserver(self)
        if TunnelManager.shared.isConfigurationLoaded {
            configureScene()
        }
    }

    func showUserAccount() {
        rootContainer.showSettings(navigateTo: .account, animated: true)
    }

    private func configureScene() {
        guard !isSceneConfigured else { return }

        isSceneConfigured = true

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

        RelayCache.Tracker.shared.addObserver(self)
        NotificationManager.shared.delegate = self

        accountDataThrottling.requestUpdate(condition: .always)
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

    @objc private func sceneDidBecomeActive() {
        TunnelManager.shared.refreshTunnelStatus()

        if isSceneConfigured {
            accountDataThrottling.requestUpdate(
                condition: settingsNavController == nil
                    ? .whenCloseToExpiryAndBeyond
                    : .always
            )
        }

        RelayCache.Tracker.shared.startPeriodicUpdates()
        TunnelManager.shared.startPeriodicPrivateKeyRotation()
        AddressCache.Tracker.shared.startPeriodicUpdates()
        ShortcutsManager.shared.updateVoiceShortcuts()

        setShowsPrivacyOverlay(false)
    }

    @objc private func sceneWillResignActive() {
        RelayCache.Tracker.shared.stopPeriodicUpdates()
        TunnelManager.shared.stopPeriodicPrivateKeyRotation()
        AddressCache.Tracker.shared.stopPeriodicUpdates()

        setShowsPrivacyOverlay(true)
    }

    @objc private func sceneDidEnterBackground() {
        let appDelegate = UIApplication.shared.delegate as? AppDelegate

        appDelegate?.scheduleBackgroundTasks()
    }
}

// MARK: - UIWindowSceneDelegate

extension SceneDelegate: UIWindowSceneDelegate {
    func scene(
        _ scene: UIScene,
        willConnectTo session: UISceneSession,
        options connectionOptions: UIScene.ConnectionOptions
    ) {
        guard let windowScene = scene as? UIWindowScene else { return }

        setupScene(windowFactory: SceneWindowFactory(windowScene: windowScene))
    }

    func sceneDidDisconnect(_ scene: UIScene) {
        // no-op
    }

    func sceneDidBecomeActive(_ scene: UIScene) {
        sceneDidBecomeActive()
    }

    func sceneWillResignActive(_ scene: UIScene) {
        sceneWillResignActive()
    }

    func sceneWillEnterForeground(_ scene: UIScene) {
        // no-op
    }

    func sceneDidEnterBackground(_ scene: UIScene) {
        sceneDidEnterBackground()
    }
}

// MARK: - SettingsButtonInteractionDelegate

protocol SettingsButtonInteractionDelegate: AnyObject {
    func viewController(
        _ controller: UIViewController,
        didRequestSettingsButtonEnabled isEnabled: Bool
    )
}

extension SceneDelegate: SettingsButtonInteractionDelegate {
    func viewController(
        _ controller: UIViewController,
        didRequestSettingsButtonEnabled isEnabled: Bool
    ) {
        setEnableSettingsButton(isEnabled: isEnabled, from: controller)
    }
}

// MARK: - RootContainerViewControllerDelegate

extension SceneDelegate: RootContainerViewControllerDelegate {
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
            accountDataThrottling.requestUpdate(condition: .always)

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
        guard TunnelManager.shared.deviceState.isLoggedIn else { return false }

        switch TunnelManager.shared.tunnelStatus.state {
        case .connected, .connecting, .reconnecting, .waitingForConnectivity:
            TunnelManager.shared.reconnectTunnel(selectNewRelay: true)
        case .disconnecting, .disconnected:
            TunnelManager.shared.startTunnel()
        case .pendingReconnect:
            break
        }
        return true
    }
}

extension SceneDelegate {
    private func setupPadUI() {
        let tunnelManager = TunnelManager.shared
        let selectLocationController = makeSelectLocationController()
        let connectController = makeConnectViewController()

        let splitViewController = CustomSplitViewController()
        splitViewController.delegate = self
        splitViewController.minimumPrimaryColumnWidth = UIMetrics.minimumSplitViewSidebarWidth
        splitViewController.preferredPrimaryColumnWidthFraction = UIMetrics
            .maximumSplitViewSidebarWidthFraction
        splitViewController.primaryEdge = .trailing
        splitViewController.dividerColor = UIColor.MainSplitView.dividerColor
        splitViewController.viewControllers = [selectLocationController, connectController]

        selectLocationViewController = selectLocationController
        self.splitViewController = splitViewController
        self.connectController = connectController

        rootContainer.setViewControllers([splitViewController], animated: false)
        showSplitViewMaster(tunnelManager.deviceState.isLoggedIn, animated: false)

        modalRootContainer.delegate = self

        let showNextController = { [weak self] (animated: Bool) in
            guard let self = self else { return }

            lazy var viewControllers: [UIViewController] = [self.makeLoginController()]

            switch tunnelManager.deviceState {
            case .loggedIn:
                let didDismissModalRoot = {
                    self.handleExpiredAccount()
                }

                self.modalRootContainer.setViewControllers(
                    viewControllers,
                    animated: self.isModalRootPresented && animated
                )

                // Dismiss modal root container if needed before proceeding.
                if self.isModalRootPresented {
                    self.modalRootContainer.dismiss(
                        animated: animated,
                        completion: didDismissModalRoot
                    )
                } else {
                    didDismissModalRoot()
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
        modalRootContainer.preferredContentSize = CGSize(width: 480, height: 600)
        modalRootContainer.modalPresentationStyle = .formSheet
        modalRootContainer.presentationController?.delegate = self
        modalRootContainer.isModalInPresentation = true

        if modalRootContainer.presentingViewController == nil {
            rootContainer.present(modalRootContainer, animated: animated)
        }
    }

    private var isModalRootPresented: Bool {
        return modalRootContainer.presentingViewController != nil
    }

    private func setupPhoneUI() {
        let showNextController = { [weak self] (animated: Bool) in
            guard let self = self else { return }

            var viewControllers: [UIViewController] = [self.makeLoginController()]

            switch TunnelManager.shared.deviceState {
            case .loggedIn:
                let connectController = self.makeConnectViewController()
                self.connectController = connectController
                viewControllers.append(connectController)

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
        let navController = SettingsNavigationController()
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
        let viewController = OutOfTimeViewController()
        viewController.delegate = self
        return viewController
    }

    private func makeConnectViewController() -> ConnectViewController {
        let connectController = ConnectViewController()
        connectController.delegate = self

        return connectController
    }

    private func makeSelectLocationController() -> SelectLocationViewController {
        let selectLocationController = SelectLocationViewController()
        selectLocationController.delegate = self

        if let cachedRelays = try? RelayCache.Tracker.shared.getCachedRelays() {
            selectLocationController.setCachedRelays(cachedRelays)
        }

        let relayConstraints = TunnelManager.shared.settings.relayConstraints

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
        let controller = RevokedDeviceViewController()
        controller.delegate = self
        return controller
    }

    private func makeLoginController() -> LoginViewController {
        let controller = LoginViewController()
        controller.delegate = self
        return controller
    }

    private func handleExpiredAccount() {
        guard case let .loggedIn(accountData, _) = TunnelManager.shared.deviceState,
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
        connectController?.setMainContentHidden(!show, animated: animated)
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
}

// MARK: - LoginViewControllerDelegate

extension SceneDelegate: LoginViewControllerDelegate {
    func loginViewController(
        _ controller: LoginViewController,
        shouldHandleLoginAction action: LoginAction,
        completion: @escaping (OperationCompletion<StoredAccountData?, Error>) -> Void
    ) {
        setEnableSettingsButton(isEnabled: false, from: controller)

        TunnelManager.shared.setAccount(action: action.setAccountAction) { operationCompletion in
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
                        interactor: DeviceManagementInteractor(accountNumber: accountNumber)
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
        rootContainer.removeSettingsButtonFromPresentationContainer()
        setEnableSettingsButton(isEnabled: true, from: controller)

        let relayConstraints = TunnelManager.shared.settings.relayConstraints
        selectLocationViewController?.setSelectedRelayLocation(
            relayConstraints.location.value,
            animated: false,
            scrollPosition: .middle
        )

        switch UIDevice.current.userInterfaceIdiom {
        case .phone:
            let connectController = makeConnectViewController()
            self.connectController = connectController
            var viewControllers = rootContainer.viewControllers
            viewControllers.append(connectController)
            rootContainer.setViewControllers(viewControllers, animated: true)
            handleExpiredAccount()
        case .pad:
            showSplitViewMaster(true, animated: true)

            controller.dismiss(animated: true) {
                self.handleExpiredAccount()
            }
        default:
            fatalError()
        }
    }

    private func setUpOutOfTimeTimer() {
        outOfTimeTimer?.invalidate()

        guard case let .loggedIn(accountData, _) = TunnelManager.shared.deviceState,
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
}

// MARK: - DeviceManagementViewControllerDelegate

extension SceneDelegate: DeviceManagementViewControllerDelegate {
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
}

// MARK: - SettingsNavigationControllerDelegate

extension SceneDelegate: SettingsNavigationControllerDelegate {
    func settingsNavigationController(
        _ controller: SettingsNavigationController,
        willNavigateTo route: SettingsNavigationRoute
    ) {
        switch route {
        case .root, .account:
            accountDataThrottling.requestUpdate(condition: .always)

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
}

// MARK: - ConnectViewControllerDelegate

extension SceneDelegate: ConnectViewControllerDelegate {
    func connectViewControllerShouldShowSelectLocationPicker(_ controller: ConnectViewController) {
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
}

// MARK: - NotificationManagerDelegate

extension SceneDelegate: NotificationManagerDelegate {
    func notificationManagerDidUpdateInAppNotifications(
        _ manager: NotificationManager,
        notifications: [InAppNotificationDescriptor]
    ) {
        connectController?.notificationController.setNotifications(notifications, animated: true)
    }
}

// MARK: - SelectLocationViewControllerDelegate

extension SceneDelegate: SelectLocationViewControllerDelegate {
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

        TunnelManager.shared.setRelayConstraints(relayConstraints) {
            TunnelManager.shared.startTunnel()
        }
    }
}

// MARK: - RevokedDeviceViewControllerDelegate

extension SceneDelegate: RevokedDeviceViewControllerDelegate {
    func revokedDeviceControllerDidRequestLogout(_ controller: RevokedDeviceViewController) {
        TunnelManager.shared.unsetAccount { [weak self] in
            self?.showLoginViewAfterLogout(dismissController: nil)
        }
    }
}

// MARK: - UIAdaptivePresentationControllerDelegate

extension SceneDelegate: UIAdaptivePresentationControllerDelegate {
    func adaptivePresentationStyle(
        for controller: UIPresentationController,
        traitCollection: UITraitCollection
    ) -> UIModalPresentationStyle {
        if controller.presentedViewController is RootContainerViewController {
            return traitCollection.horizontalSizeClass == .regular ? .formSheet : .fullScreen
        } else {
            return .none
        }
    }

    func presentationController(
        _ presentationController: UIPresentationController,
        willPresentWithAdaptiveStyle style: UIModalPresentationStyle,
        transitionCoordinator: UIViewControllerTransitionCoordinator?
    ) {
        let actualStyle: UIModalPresentationStyle

        // When adaptive presentation is not changing, the `style` is set to `.none`
        if case .none = style {
            actualStyle = presentationController.presentedViewController.modalPresentationStyle
        } else {
            actualStyle = style
        }

        // Force hide header bar in .formSheet presentation and show it in .fullScreen presentation
        if let wrapper = presentationController
            .presentedViewController as? RootContainerViewController
        {
            wrapper.setOverrideHeaderBarHidden(actualStyle == .formSheet, animated: false)
        }

        guard actualStyle == .formSheet else {
            // Move the settings button back into header bar
            rootContainer.removeSettingsButtonFromPresentationContainer()

            return
        }

        // Add settings button into the modal container to make it accessible by user
        if let transitionCoordinator = transitionCoordinator {
            transitionCoordinator.animate { context in
                self.rootContainer.addSettingsButtonToPresentationContainer(context.containerView)
            }
        } else if let containerView = presentationController.containerView {
            rootContainer.addSettingsButtonToPresentationContainer(containerView)
        } else {
            logger.warning(
                """
                Cannot obtain the containerView for presentation controller when presenting with \
                adaptive style \(actualStyle.rawValue) and missing transition coordinator.
                """
            )
        }
    }
}

// MARK: - TunnelObserver

extension SceneDelegate: TunnelObserver {
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
            accountDataThrottling.reset()

        case .revoked:
            accountDataThrottling.reset()
            showRevokedDeviceView()
        }
    }

    func tunnelManager(_ manager: TunnelManager, didFailWithError error: Error) {
        // no-op
    }
}

// MARK: - RelayCacheObserver

extension SceneDelegate: RelayCacheObserver {
    func relayCache(
        _ relayCache: RelayCache.Tracker,
        didUpdateCachedRelays cachedRelays: RelayCache.CachedRelays
    ) {
        selectLocationViewController?.setCachedRelays(cachedRelays)
    }
}

// MARK: - UISplitViewControllerDelegate

extension SceneDelegate: UISplitViewControllerDelegate {
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
        return connectController
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
}

// MARK: - Window factory

protocol WindowFactory {
    func create() -> UIWindow
}

struct ClassicWindowFactory: WindowFactory {
    func create() -> UIWindow {
        return UIWindow(frame: UIScreen.main.bounds)
    }
}

struct SceneWindowFactory: WindowFactory {
    let windowScene: UIWindowScene

    func create() -> UIWindow {
        return UIWindow(windowScene: windowScene)
    }
}
