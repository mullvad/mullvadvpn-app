//
//  SceneDelegate.swift
//  MullvadVPN
//
//  Created by pronebird on 20/05/2022.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import MullvadLogging
import MullvadREST
import MullvadSettings
import MullvadTypes
import Network
import Operations
import Routing
import UIKit

class SceneDelegate: UIResponder, UIWindowSceneDelegate, @preconcurrency SettingsMigrationUIHandler {
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
                outgoingConnectionProxy: OutgoingConnectionProxy(
                    urlSession: REST.makeURLSession(addressCache: appDelegate.addressCache),
                    hostname: ApplicationConfiguration.hostName
                )
            ),
            appPreferences: appDelegate.appPreferences,
            accessMethodRepository: accessMethodRepository,
            transportProvider: appDelegate.configuredTransportProvider,
            ipOverrideRepository: appDelegate.ipOverrideRepository,
            relaySelectorWrapper: appDelegate.relaySelector
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

    // swiftlint:disable:next function_body_length
    func scene(_ scene: UIScene, openURLContexts URLContexts: Set<UIOpenURLContext>) {
        do {
            let url = URLContexts.first!.url

            guard
                let components = NSURLComponents(url: url, resolvingAgainstBaseURL: true),
                let albumPath = components.path,
                let params = components.queryItems
            else {
                throw NSError(domain: "", code: 1, userInfo: nil)
            }

            var currentSettings = tunnelManager.settings

            switch albumPath {
            case "settings":
                try params.forEach { param in
                    switch param.name {
                    case "daita":
                        currentSettings.daita.daitaState = param.value == "on" ? .on : .off
                    case "directOnly":
                        currentSettings.daita.directOnlyState = param.value == "on" ? .on : .off
                    case "multihop":
                        currentSettings.tunnelMultihopState = param.value == "on" ? .on : .off
                    case "quantumResistance":
                        currentSettings.tunnelQuantumResistance = param.value == "on" ? .on : .off
                    case "obfuscation":
                        var state: WireGuardObfuscationState = .automatic
                        var port: Int?

                        let components = param.value!.split(separator: ",")

                        try components.forEach { component in
                            let keyValue = component.split(separator: "=")

                            switch keyValue.first! {
                            case "state":
                                switch keyValue.last! {
                                case "automatic":
                                    state = .automatic
                                case "off":
                                    state = .off
                                case "shadowsocks":
                                    state = .shadowsocks
                                case "udpOverTcp":
                                    state = .udpOverTcp
                                default:
                                    throw NSError(domain: "", code: 2, userInfo: nil)
                                }
                            case "port":
                                port = Int(keyValue.last!)
                            default:
                                throw NSError(domain: "", code: 3, userInfo: nil)
                            }
                        }

                        currentSettings.wireGuardObfuscation.state = state

                        switch state {
                        case .shadowsocks:
                            let shadowSocksPort = port.flatMap {
                                WireGuardObfuscationShadowsocksPort.custom(UInt16($0))
                            } ?? WireGuardObfuscationShadowsocksPort.automatic

                            currentSettings.wireGuardObfuscation.shadowsocksPort = shadowSocksPort
                        case .udpOverTcp:
                            let udpTcpPort: WireGuardObfuscationUdpOverTcpPort = switch port {
                            case nil:
                                .automatic
                            case 80:
                                .port80
                            case 5001:
                                .port5001
                            default:
                                throw NSError(domain: "", code: 4, userInfo: nil)
                            }

                            currentSettings.wireGuardObfuscation.udpOverTcpPort = udpTcpPort
                        default:
                            break
                        }
                    default:
                        throw NSError(domain: "", code: 5, userInfo: nil)
                    }
                }
            case "ipOverrides":
                var overrides: [IPOverride] = []

                try params.forEach { param in
                    var hostname = ""
                    var ipv4: IPv4Address?
                    var ipv6: IPv6Address?

                    let hostComponents = param.value!.split(separator: ",")

                    try hostComponents.forEach { component in
                        let keyValue = component.split(separator: "=")

                        switch keyValue.first! {
                        case "hostname":
                            hostname = String(keyValue.last!)
                        case "ipv4_addr_in":
                            ipv4 = keyValue.last.flatMap { IPv4Address(String($0)) }
                        case "ipv6_addr_in":
                            ipv6 = keyValue.last.flatMap { IPv6Address(String($0)) }
                        default:
                            throw NSError(domain: "", code: 6, userInfo: nil)
                        }
                    }

                    try overrides.append(IPOverride(hostname: hostname, ipv4Address: ipv4, ipv6Address: ipv6))
                }

                let interactor = IPOverrideInteractor(repository: IPOverrideRepository(), tunnelManager: tunnelManager)
                try interactor.handleImport(of: JSONEncoder().encode(RelayOverrides(overrides: overrides)), context: .text)
            default:
                throw NSError(domain: "", code: 7, userInfo: nil)
            }

            tunnelManager.updateSettings(currentSettings)

            let presentation = AlertPresentation(
                id: "import-successful",
                icon: .info,
                title: NSLocalizedString(
                    "SHARE_IMPORT_SUCCESSFUL_TITLE",
                    tableName: "Settings",
                    value: "Success!",
                    comment: ""
                ),
                message: NSLocalizedString(
                    "SHARE_IMPORT_SUCCESSFUL_MESSAGE",
                    tableName: "Settings",
                    value: "The new settings were successfully applied.",
                    comment: ""
                ),
                buttons: [
                    AlertAction(title: "Got it!", style: .default)
                ]
            )

            let alert = AlertPresenter(context: appCoordinator)
            alert.showAlert(presentation: presentation, animated: true)
        } catch {
            print(error)

            let presentation = AlertPresentation(
                id: "import-unsuccessful",
                icon: .warning,
                title: NSLocalizedString(
                    "SHARE_IMPORT_SUCCESSFUL_TITLE",
                    tableName: "Settings",
                    value: "Failure!",
                    comment: ""
                ),
                message: NSLocalizedString(
                    "SHARE_IMPORT_SUCCESSFUL_MESSAGE",
                    tableName: "Settings",
                    value: "The new settings could not be applied.",
                    comment: ""
                ),
                buttons: [
                    AlertAction(title: "Got it!", style: .default)
                ]
            )

            let alert = AlertPresenter(context: appCoordinator)
            alert.showAlert(presentation: presentation, animated: true)
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
