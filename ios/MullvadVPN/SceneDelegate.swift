//
//  SceneDelegate.swift
//  MullvadVPN
//
//  Created by pronebird on 20/05/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import MullvadLogging
import MullvadREST
import MullvadSettings
import MullvadTypes
import Operations
import UIKit

class SceneDelegate: UIResponder, UIWindowSceneDelegate, SettingsMigrationUIHandler {
    private let logger = Logger(label: "SceneDelegate")

    var window: UIWindow?
    private var privacyOverlayWindow: UIWindow?
    private var isSceneConfigured = false

    private var appCoordinator: ApplicationCoordinator?
    private var accountDataThrottling: AccountDataThrottling?
    private var deviceDataThrottling: DeviceDataThrottling?

    private var tunnelObserver: TunnelObserver?

    private var appDelegate: AppDelegate {
        // swiftlint:disable:next force_cast
        UIApplication.shared.delegate as! AppDelegate
    }

    private var accessMethodRepository: AccessMethodRepositoryProtocol {
        appDelegate.accessMethodRepository
    }

    private var tunnelManager: TunnelManager {
        appDelegate.tunnelManager
    }

    // MARK: - Private

    private func addTunnelObserver() {
        let tunnelObserver = TunnelBlockObserver(
            didLoadConfiguration: { [weak self] _ in
                self?.configureScene()
            },
            didUpdateDeviceState: { [weak self] _, deviceState, _ in
                self?.deviceStateDidChange(deviceState)
            }
        )

        self.tunnelObserver = tunnelObserver

        tunnelManager.addObserver(tunnelObserver)
    }

    private func configureScene() {
        guard !isSceneConfigured else { return }

        isSceneConfigured = true

        accountDataThrottling = AccountDataThrottling(tunnelManager: tunnelManager)
        deviceDataThrottling = DeviceDataThrottling(tunnelManager: tunnelManager)
        refreshLoginMetadata(forceUpdate: true)

        appCoordinator = ApplicationCoordinator(
            tunnelManager: tunnelManager,
            storePaymentManager: appDelegate.storePaymentManager,
            relayCacheTracker: appDelegate.relayCacheTracker,
            apiProxy: appDelegate.apiProxy,
            devicesProxy: appDelegate.devicesProxy,
            accountsProxy: appDelegate.accountsProxy,
            outgoingConnectionService: OutgoingConnectionService(
                outgoingConnectionProxy: OutgoingConnectionProxy(urlSession: URLSession(configuration: .ephemeral))
            ),
            appPreferences: AppPreferences(),
            accessMethodRepository: accessMethodRepository
        )

        appCoordinator?.onShowSettings = { [weak self] in
            // Refresh account data and device each time user opens settings
            self?.refreshLoginMetadata(forceUpdate: true)
        }

        appCoordinator?.onShowAccount = { [weak self] in
            // Refresh account data and device each time user opens account controller
            self?.refreshLoginMetadata(forceUpdate: true)
        }

        window?.rootViewController = appCoordinator?.rootViewController
        appCoordinator?.start()
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

    private func deviceStateDidChange(_ deviceState: DeviceState) {
        switch deviceState {
        case .loggedOut, .revoked:
            resetLoginMetadataThrottling()

        case .loggedIn:
            break
        }
    }

    /**
     Refresh login metadata (account and device data) potentially throttling refresh requests based on recency of
     the last issued request.

     Account data is always refreshed when either settings or account are presented on screen, otherwise only when close
     to or past expiry.

     Both account and device data are refreshed regardless of other conditions when `forceUpdate` is `true`.

     For more information on exact timings used for throttling refresh requests refer to `AccountDataThrottling` and
     `DeviceDataThrottling` types.
     */
    private func refreshLoginMetadata(forceUpdate: Bool) {
        let condition: AccountDataThrottling.Condition

        if forceUpdate {
            condition = .always
        } else {
            let isPresentingSettings = appCoordinator?.isPresentingSettings ?? false
            let isPresentingAccount = appCoordinator?.isPresentingAccount ?? false

            condition = isPresentingSettings || isPresentingAccount ? .always : .whenCloseToExpiryAndBeyond
        }

        accountDataThrottling?.requestUpdate(condition: condition)
        deviceDataThrottling?.requestUpdate(forceUpdate: forceUpdate)
    }

    /**
     Reset throttling for login metadata making a subsequent refresh request execute unthrottled.
     */
    private func resetLoginMetadataThrottling() {
        accountDataThrottling?.reset()
        deviceDataThrottling?.reset()
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

        addTunnelObserver()

        if tunnelManager.isConfigurationLoaded {
            configureScene()
        }
    }

    func sceneDidDisconnect(_ scene: UIScene) {}

    func sceneDidBecomeActive(_ scene: UIScene) {
        if isSceneConfigured {
            refreshLoginMetadata(forceUpdate: false)
        }

        setShowsPrivacyOverlay(false)
    }

    func sceneWillResignActive(_ scene: UIScene) {
        setShowsPrivacyOverlay(true)
    }

    func sceneWillEnterForeground(_ scene: UIScene) {}

    func sceneDidEnterBackground(_ scene: UIScene) {}

    // MARK: - SettingsMigrationUIHandler

    func showMigrationError(_ error: Error, completionHandler: @escaping () -> Void) {
        guard let appCoordinator else {
            completionHandler()
            return
        }

        let presentation = AlertPresentation(
            id: "settings-migration-error-alert",
            title: NSLocalizedString(
                "ALERT_TITLE",
                tableName: "SettingsMigrationUI",
                value: "Settings migration error",
                comment: ""
            ),
            message: Self.migrationErrorReason(error),
            buttons: [
                AlertAction(
                    title: NSLocalizedString("Got it!", tableName: "SettingsMigrationUI", comment: ""),
                    style: .default,
                    handler: {
                        completionHandler()
                    }
                ),
            ]
        )

        let presenter = AlertPresenter(context: appCoordinator)
        presenter.showAlert(presentation: presentation, animated: true)
    }

    private static func migrationErrorReason(_ error: Error) -> String {
        if error is UnsupportedSettingsVersionError {
            return NSLocalizedString(
                "NEWER_STORED_SETTINGS_ERROR",
                tableName: "SettingsMigrationUI",
                value: """
                The version of settings stored on device is unrecognized.\
                Settings will be reset to defaults and the device will be logged out.
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
