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
        return UIApplication.shared.delegate as! AppDelegate
    }

    private var tunnelManager: TunnelManager {
        return appDelegate.tunnelManager
    }

    // MARK: - Deep link

    func showUserAccount() {
        appCoordinator?.showAccount()
    }

    // MARK: - Private

    private func addTunnelObserver() {
        let tunnelObserver = TunnelBlockObserver(
            didLoadConfiguration: { [weak self] _ in
                self?.configureScene()
            },
            didUpdateDeviceState: { [weak self] _, deviceState in
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
        refreshDeviceAndAccountData(forceUpdate: true)

        appCoordinator = ApplicationCoordinator(
            tunnelManager: tunnelManager,
            storePaymentManager: appDelegate.storePaymentManager,
            relayCacheTracker: appDelegate.relayCacheTracker,
            apiProxy: appDelegate.apiProxy,
            devicesProxy: appDelegate.devicesProxy
        )

        appCoordinator?.onShowSettings = { [weak self] in
            // Refresh account data each time user opens settings
            self?.refreshDeviceAndAccountData(forceUpdate: true)
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
        case .loggedOut:
            resetDeviceAndAccountDataThrottling()

        case .revoked:
            resetDeviceAndAccountDataThrottling()

        case .loggedIn:
            break
        }
    }

    private func refreshDeviceAndAccountData(forceUpdate: Bool) {
        let isPresentingSettings = appCoordinator?.isPresentingSettings ?? false

        let condition: AccountDataThrottling.Condition

        if forceUpdate {
            condition = .always
        } else {
            condition = isPresentingSettings ? .always : .whenCloseToExpiryAndBeyond
        }

        accountDataThrottling?.requestUpdate(condition: condition)
        deviceDataThrottling?.requestUpdate(forceUpdate: forceUpdate)
    }

    private func resetDeviceAndAccountDataThrottling() {
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
            refreshDeviceAndAccountData(forceUpdate: false)
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
