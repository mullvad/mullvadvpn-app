//
//  SceneDelegate.swift
//  MullvadVPN
//
//  Created by pronebird on 20/05/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import UIKit
import Logging

class SceneDelegate: UIResponder {
    private let logger = Logger(label: "SceneDelegate")

    var window: UIWindow?
    private var privacyOverlayWindow: UIWindow?
    private var isSceneConfigured = false

    private let rootContainer = RootContainerViewController()
    private var splitViewController: CustomSplitViewController?
    private var selectLocationViewController: SelectLocationViewController?
    private var connectController: ConnectViewController?
    private weak var settingsNavController: SettingsNavigationController?

    override init() {
        super.init()

        addSceneEvents()
    }

    func setupScene(windowFactory: WindowFactory) {
        window = windowFactory.create()
        window?.rootViewController = LaunchViewController()

        privacyOverlayWindow = windowFactory.create()
        privacyOverlayWindow?.rootViewController = LaunchViewController()
        privacyOverlayWindow?.windowLevel = .alert + 1

        window?.makeKeyAndVisible()

        TunnelManager.shared.addObserver(self)
        if TunnelManager.shared.isLoadedConfiguration {
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

    private func addSceneEvents() {
        if #available(iOS 13, *) {
            // no-op
        } else {
            let notificationCenter = NotificationCenter.default

            notificationCenter.addObserver(
                self,
                selector: #selector(sceneDidBecomeActive),
                name: UIApplication.didBecomeActiveNotification,
                object: nil
            )
            notificationCenter.addObserver(
                self,
                selector: #selector(sceneDidEnterBackground),
                name: UIApplication.didEnterBackgroundNotification,
                object: nil
            )
            notificationCenter.addObserver(
                self,
                selector: #selector(sceneWillResignActive),
                name: UIApplication.willResignActiveNotification,
                object: nil
            )
        }
    }

    @objc private func sceneDidBecomeActive() {
        TunnelManager.shared.refreshTunnelStatus()

        RelayCache.Tracker.shared.startPeriodicUpdates()
        TunnelManager.shared.startPeriodicPrivateKeyRotation()
        AddressCache.Tracker.shared.startPeriodicUpdates()

        setShowsPrivacyOverlay(false)
    }

    @objc private func sceneWillResignActive() {
        RelayCache.Tracker.shared.stopPeriodicUpdates()
        TunnelManager.shared.stopPeriodicPrivateKeyRotation()
        AddressCache.Tracker.shared.stopPeriodicUpdates()

        setShowsPrivacyOverlay(true)
    }

    @objc private func sceneDidEnterBackground() {
        if #available(iOS 13, *) {
            let appDelegate = UIApplication.shared.delegate as? AppDelegate

            appDelegate?.scheduleBackgroundTasks()
        }
    }
}

// MARK: - UIWindowSceneDelegate

@available(iOS 13.0, *)
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

// MARK: - RootContainerViewControllerDelegate

extension SceneDelegate: RootContainerViewControllerDelegate {
    func rootContainerViewControllerShouldShowSettings(_ controller: RootContainerViewController, navigateTo route: SettingsNavigationRoute?, animated: Bool) {
        // Check if settings controller is already presented.
        if let settingsNavController = settingsNavController {
            if let route = route {
                settingsNavController.navigate(to: route, animated: animated)
            } else {
                settingsNavController.popToRootViewController(animated: animated)
            }
        } else {
            let navController = makeSettingsNavigationController(route: route)

            // On iPad the login controller can be presented modally above the root container.
            // in that case we have to use the presented controller to present the next modal.
            if let presentedController = controller.presentedViewController {
                presentedController.present(navController, animated: true)
            } else {
                controller.present(navController, animated: true)
            }

            // Save the reference for later.
            self.settingsNavController = navController
        }
    }

    func rootContainerViewSupportedInterfaceOrientations(_ controller: RootContainerViewController) -> UIInterfaceOrientationMask {
        switch UIDevice.current.userInterfaceIdiom {
        case .pad:
            return [.landscape, .portrait]
        case .phone:
            return [.portrait]
        default:
            return controller.supportedInterfaceOrientations
        }
    }

    func rootContainerViewAccessibilityPerformMagicTap(_ controller: RootContainerViewController) -> Bool {
        guard TunnelManager.shared.isAccountSet else { return false }

        switch TunnelManager.shared.tunnelState {
        case .connected, .connecting, .reconnecting:
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
        let selectLocationController = makeSelectLocationController()
        let connectController = makeConnectViewController()

        let splitViewController = CustomSplitViewController()
        splitViewController.delegate = self
        splitViewController.minimumPrimaryColumnWidth = UIMetrics.minimumSplitViewSidebarWidth
        splitViewController.preferredPrimaryColumnWidthFraction = UIMetrics.maximumSplitViewSidebarWidthFraction
        splitViewController.primaryEdge = .trailing
        splitViewController.dividerColor = UIColor.MainSplitView.dividerColor
        splitViewController.viewControllers = [selectLocationController, connectController]

        self.selectLocationViewController = selectLocationController
        self.splitViewController = splitViewController
        self.connectController = connectController

        rootContainer.setViewControllers([splitViewController], animated: false)
        showSplitViewMaster(TunnelManager.shared.isAccountSet, animated: false)

        let rootContainerWrapper = makeLoginContainerController()

        if !TermsOfService.isAgreed {
            let termsOfServiceViewController = self.makeTermsOfServiceController { [weak self] viewController in
                guard let self = self else { return }

                if TunnelManager.shared.isAccountSet {
                    rootContainerWrapper.dismiss(animated: true) {
                        self.showAccountSettingsControllerIfAccountExpired()
                    }
                } else {
                    rootContainerWrapper.pushViewController(self.makeLoginController(), animated: true)
                }
            }
            rootContainerWrapper.setViewControllers([termsOfServiceViewController], animated: false)
            rootContainer.present(rootContainerWrapper, animated: false)
        } else if !TunnelManager.shared.isAccountSet {
            rootContainerWrapper.setViewControllers([makeLoginController()], animated: false)
            rootContainer.present(rootContainerWrapper, animated: false)
        } else {
            self.showAccountSettingsControllerIfAccountExpired()
        }
    }

    private func setupPhoneUI() {
        let showNextController = { [weak self] (animated: Bool) in
            guard let self = self else { return }

            let loginViewController = self.makeLoginController()
            var viewControllers: [UIViewController] = [loginViewController]

            if TunnelManager.shared.isAccountSet {
                let connectController = self.makeConnectViewController()
                viewControllers.append(connectController)
                self.connectController = connectController
            }

            self.rootContainer.setViewControllers(viewControllers, animated: animated) {
                self.showAccountSettingsControllerIfAccountExpired()
            }
        }

        if TermsOfService.isAgreed {
            showNextController(false)
        } else {
            let termsOfServiceController = self.makeTermsOfServiceController { _ in
                showNextController(true)
            }

            rootContainer.setViewControllers([termsOfServiceController], animated: false)
        }
    }

    private func makeSettingsNavigationController(route: SettingsNavigationRoute?) -> SettingsNavigationController {
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

        let relayConstraints = TunnelManager.shared.tunnelSettings?.relayConstraints
        if let relayLocation = relayConstraints?.location.value {
            selectLocationController.setSelectedRelayLocation(
                relayLocation,
                animated: false,
                scrollPosition: .middle
            )
        }

        return selectLocationController
    }

    private func makeTermsOfServiceController(
        completion: @escaping (UIViewController) -> Void
    ) -> TermsOfServiceViewController
    {
        let controller = TermsOfServiceViewController()

        if UIDevice.current.userInterfaceIdiom == .pad {
            controller.modalPresentationStyle = .formSheet
            if #available(iOS 13.0, *) {
                controller.isModalInPresentation = true
            }
        }

        controller.completionHandler = { controller in
            TermsOfService.setAgreed()
            completion(controller)
        }

        return controller
    }

    private func makeLoginContainerController() -> RootContainerViewController {
        let rootContainerWrapper = RootContainerViewController()
        rootContainerWrapper.delegate = self
        rootContainerWrapper.preferredContentSize = CGSize(width: 480, height: 600)

        if UIDevice.current.userInterfaceIdiom == .pad {
            rootContainerWrapper.modalPresentationStyle = .formSheet
            if #available(iOS 13.0, *) {
                // Prevent swiping off the login or terms of service controllers
                rootContainerWrapper.isModalInPresentation = true
            }
        }

        rootContainerWrapper.presentationController?.delegate = self

        return rootContainerWrapper
    }

    private func makeLoginController() -> LoginViewController {
        let controller = LoginViewController()
        controller.delegate = self
        return controller
    }

    private func showAccountSettingsControllerIfAccountExpired() {
        guard let accountExpiry = TunnelManager.shared.accountExpiry, accountExpiry <= Date() else { return }

        rootContainer.showSettings(navigateTo: .account, animated: true)
    }

    private func showSplitViewMaster(_ show: Bool, animated: Bool) {
        splitViewController?.preferredDisplayMode = show ? .allVisible : .primaryHidden
        connectController?.setMainContentHidden(!show, animated: animated)
    }
}

// MARK: - LoginViewControllerDelegate

extension SceneDelegate: LoginViewControllerDelegate {

    func loginViewController(_ controller: LoginViewController, loginWithAccountToken accountNumber: String, completion: @escaping (OperationCompletion<StoredAccountData?, TunnelManager.Error>) -> Void) {
        rootContainer.setEnableSettingsButton(false)

        TunnelManager.shared.setAccount(action: .existing(accountNumber)) { operationCompletion in
            switch operationCompletion {
            case .success:
                self.logger.debug("Logged in with existing account.")
                // RootContainer's settings button will be re-enabled in `loginViewControllerDidLogin`

            case .failure(let error):
                self.logger.error(chainedError: error, message: "Failed to log in with existing account.")
                fallthrough

            case .cancelled:
                self.rootContainer.setEnableSettingsButton(true)
            }

            completion(operationCompletion)
        }
    }

    func loginViewControllerLoginWithNewAccount(_ controller: LoginViewController, completion: @escaping (OperationCompletion<StoredAccountData?, TunnelManager.Error>) -> Void) {
        rootContainer.setEnableSettingsButton(false)

        TunnelManager.shared.setAccount(action: .new) { operationCompletion in
            switch operationCompletion {
            case .success:
                self.logger.debug("Logged in with new account number.")
                // RootContainer's settings button will be re-enabled in `loginViewControllerDidLogin`

            case .failure(let error):
                self.logger.error(chainedError: error, message: "Failed to log in with new account.")
                fallthrough

            case .cancelled:
                self.rootContainer.setEnableSettingsButton(true)
            }

            completion(operationCompletion)
        }
    }

    func loginViewControllerDidLogin(_ controller: LoginViewController) {
        window?.isUserInteractionEnabled = false

        // Move the settings button back into header bar
        rootContainer.removeSettingsButtonFromPresentationContainer()

        let relayConstraints = TunnelManager.shared.tunnelSettings?.relayConstraints
        self.selectLocationViewController?.setSelectedRelayLocation(
            relayConstraints?.location.value,
            animated: false,
            scrollPosition: .middle
        )

        switch UIDevice.current.userInterfaceIdiom {
        case .phone:
            let connectController = self.makeConnectViewController()
            rootContainer.pushViewController(connectController, animated: true) {
                self.showAccountSettingsControllerIfAccountExpired()
            }
            self.connectController = connectController
        case .pad:
            self.showSplitViewMaster(true, animated: true)

            controller.dismiss(animated: true) {
                self.showAccountSettingsControllerIfAccountExpired()
            }
        default:
            fatalError()
        }

        window?.isUserInteractionEnabled = true
        rootContainer.setEnableSettingsButton(true)
    }

}

// MARK: - SettingsNavigationControllerDelegate

extension SceneDelegate: SettingsNavigationControllerDelegate {

    func settingsNavigationController(_ controller: SettingsNavigationController, didFinishWithReason reason: SettingsDismissReason) {
        switch UIDevice.current.userInterfaceIdiom {
        case .phone:
            if case .userLoggedOut = reason {
                rootContainer.popToRootViewController(animated: false)

                let loginController = rootContainer.topViewController as? LoginViewController

                loginController?.reset()
            }
            controller.dismiss(animated: true)

        case .pad:
            if case .userLoggedOut = reason {
                self.showSplitViewMaster(false, animated: true)
            }

            controller.dismiss(animated: true) {
                if case .userLoggedOut = reason {
                    let rootContainerWrapper = self.makeLoginContainerController()
                    rootContainerWrapper.setViewControllers([self.makeLoginController()], animated: false)
                    self.rootContainer.present(rootContainerWrapper, animated: true)
                }
            }

        default:
            fatalError()
        }

    }

}

// MARK: - ConnectViewControllerDelegate

extension SceneDelegate: ConnectViewControllerDelegate {

    func connectViewControllerShouldShowSelectLocationPicker(_ controller: ConnectViewController) {
        let contentController = makeSelectLocationController()
        contentController.navigationItem.largeTitleDisplayMode = .never
        contentController.navigationItem.rightBarButtonItem = UIBarButtonItem(barButtonSystemItem: .done, target: self, action: #selector(handleDismissSelectLocationController(_:)))

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
    func notificationManagerDidUpdateInAppNotifications(_ manager: NotificationManager, notifications: [InAppNotificationDescriptor]) {
        connectController?.notificationController.setNotifications(notifications, animated: true)
    }
}

// MARK: - SelectLocationViewControllerDelegate

extension SceneDelegate: SelectLocationViewControllerDelegate {
    func selectLocationViewController(_ controller: SelectLocationViewController, didSelectRelayLocation relayLocation: RelayLocation) {
        // Dismiss view controller in modal presentation
        if controller.presentingViewController != nil {
            self.window?.isUserInteractionEnabled = false
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

        TunnelManager.shared.setRelayConstraints(relayConstraints) { error in
            if let error = error {
                self.logger.error(chainedError: error, message: "Failed to update relay constraints")
            } else {
                self.logger.debug("Updated relay constraints: \(relayConstraints)")

                TunnelManager.shared.startTunnel()
            }
        }
    }
}

// MARK: - UIAdaptivePresentationControllerDelegate

extension SceneDelegate: UIAdaptivePresentationControllerDelegate {

    func adaptivePresentationStyle(for controller: UIPresentationController, traitCollection: UITraitCollection) -> UIModalPresentationStyle {
        if controller.presentedViewController is RootContainerViewController {
            return traitCollection.horizontalSizeClass == .regular ? .formSheet : .fullScreen
        } else {
            return .none
        }
    }

    func presentationController(_ presentationController: UIPresentationController, willPresentWithAdaptiveStyle style: UIModalPresentationStyle, transitionCoordinator: UIViewControllerTransitionCoordinator?) {
        let actualStyle: UIModalPresentationStyle

        // When adaptive presentation is not changing, the `style` is set to `.none`
        if case .none = style {
            actualStyle = presentationController.presentedViewController.modalPresentationStyle
        } else {
            actualStyle = style
        }

        // Force hide header bar in .formSheet presentation and show it in .fullScreen presentation
        if let wrapper = presentationController.presentedViewController as? RootContainerViewController {
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

    func tunnelManager(_ manager: TunnelManager, didUpdateTunnelState tunnelState: TunnelState) {
        // no-op
    }

    func tunnelManager(_ manager: TunnelManager, didUpdateTunnelSettings tunnelSettings: TunnelSettingsV2?) {
        // no-op
    }

    func tunnelManager(_ manager: TunnelManager, didFailWithError error: TunnelManager.Error) {
        // no-op
    }
}

// MARK: - RelayCacheObserver

extension SceneDelegate: RelayCacheObserver {

    func relayCache(_ relayCache: RelayCache.Tracker, didUpdateCachedRelays cachedRelays: RelayCache.CachedRelays) {
        DispatchQueue.main.async {
            self.selectLocationViewController?.setCachedRelays(cachedRelays)
        }
    }

}


// MARK: - UISplitViewControllerDelegate

extension SceneDelegate: UISplitViewControllerDelegate {

    func primaryViewController(forExpanding splitViewController: UISplitViewController) -> UIViewController? {
        // Restore the select location controller as primary when expanding the split view
        return selectLocationViewController
    }

    func primaryViewController(forCollapsing splitViewController: UISplitViewController) -> UIViewController? {
        // Set the connect controller as primary when collapsing the split view
        return connectController
    }

    func splitViewController(_ splitViewController: UISplitViewController, separateSecondaryFrom primaryViewController: UIViewController) -> UIViewController? {
        // Dismiss the select location controller when expanding the split view
        if self.selectLocationViewController?.presentingViewController != nil {
            self.selectLocationViewController?.dismiss(animated: false)
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
@available(iOS 13.0, *)
struct SceneWindowFactory: WindowFactory {
    let windowScene: UIWindowScene

    func create() -> UIWindow {
        return UIWindow(windowScene: windowScene)
    }
}
