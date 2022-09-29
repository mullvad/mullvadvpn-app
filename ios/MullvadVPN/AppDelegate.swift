//
//  AppDelegate.swift
//  MullvadVPN
//
//  Created by pronebird on 19/03/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import BackgroundTasks
import Intents
import Logging
import Operations
import StoreKit
import UIKit
import UserNotifications

@UIApplicationMain
class AppDelegate: UIResponder, UIApplicationDelegate {
    private var logger: Logger!

    #if targetEnvironment(simulator)
    private let simulatorTunnelProvider = SimulatorTunnelProviderHost()
    #endif

    private let operationQueue: AsyncOperationQueue = {
        let operationQueue = AsyncOperationQueue()
        operationQueue.maxConcurrentOperationCount = 1
        return operationQueue
    }()

    // MARK: - Application lifecycle

    func application(
        _ application: UIApplication,
        didFinishLaunchingWithOptions launchOptions: [UIApplication.LaunchOptionsKey: Any]?
    ) -> Bool {
        initLoggingSystem(bundleIdentifier: Bundle.main.bundleIdentifier!)

        logger = Logger(label: "AppDelegate")

        #if targetEnvironment(simulator)
        // Configure mock tunnel provider on simulator
        SimulatorTunnelProvider.shared.delegate = simulatorTunnelProvider
        #endif

        registerBackgroundTasks()
        setupPaymentHandler()
        setupNotificationHandler()

        let setupTunnelManagerOperation = AsyncBlockOperation(dispatchQueue: .main) { operation in
            TunnelManager.shared.loadConfiguration { error in
                // TODO: avoid throwing fatal error and show the problem report UI instead.
                if let error = error {
                    fatalError(error.localizedDescription)
                }

                self.logger.debug("Finished initialization.")

                NotificationManager.shared.updateNotifications()
                AppStorePaymentManager.shared.startPaymentQueueMonitoring()

                operation.finish()
            }
        }

        operationQueue.addOperation(setupTunnelManagerOperation)

        return true
    }

    func application(_ application: UIApplication, handlerFor intent: INIntent) -> Any? {
        switch intent {
        case is StartVPNIntent:
            return StartVPNIntentHandler()
        case is StopVPNIntent:
            return StopVPNIntentHandler()
        case is ReconnectVPNIntent:
            return ReconnectVPNIntentHandler()
        default:
            return nil
        }
    }

    // MARK: - UISceneSession lifecycle

    func application(
        _ application: UIApplication,
        configurationForConnecting connectingSceneSession: UISceneSession,
        options: UIScene.ConnectionOptions
    ) -> UISceneConfiguration {
        // Called when a new scene session is being created.
        // Use this method to select a configuration to create the new scene with.
        let sceneConfiguration = UISceneConfiguration(
            name: "Default Configuration",
            sessionRole: connectingSceneSession.role
        )
        sceneConfiguration.delegateClass = SceneDelegate.self

        return sceneConfiguration
    }

    func application(
        _ application: UIApplication,
        didDiscardSceneSessions sceneSessions: Set<UISceneSession>
    ) {
        // Called when the user discards a scene session.
        // If any sessions were discarded while the application was not running,
        // this will be called shortly after application:didFinishLaunchingWithOptions.
        // Use this method to release any resources that were specific to
        // the discarded scenes, as they will not return.
    }

    // MARK: - Background tasks

    private func registerBackgroundTasks() {
        registerAppRefreshTask()
        registerAddressCacheUpdateTask()
        registerKeyRotationTask()
    }

    private func registerAppRefreshTask() {
        let isRegistered = BGTaskScheduler.shared.register(
            forTaskWithIdentifier: ApplicationConfiguration.appRefreshTaskIdentifier,
            using: nil
        ) { task in
            let handle = RelayCache.Tracker.shared.updateRelays { completion in
                task.setTaskCompleted(success: completion.isSuccess)
            }

            task.expirationHandler = {
                handle.cancel()
            }

            self.scheduleAppRefreshTask()
        }

        if isRegistered {
            logger.debug("Registered app refresh task.")
        } else {
            logger.error("Failed to register app refresh task.")
        }
    }

    private func registerKeyRotationTask() {
        let isRegistered = BGTaskScheduler.shared.register(
            forTaskWithIdentifier: ApplicationConfiguration.privateKeyRotationTaskIdentifier,
            using: nil
        ) { task in
            let handle = TunnelManager.shared.rotatePrivateKey(forceRotate: false) { completion in
                self.scheduleKeyRotationTask()

                task.setTaskCompleted(success: completion.isSuccess)
            }

            task.expirationHandler = {
                handle.cancel()
            }
        }

        if isRegistered {
            logger.debug("Registered private key rotation task.")
        } else {
            logger.error("Failed to register private key rotation task.")
        }
    }

    private func registerAddressCacheUpdateTask() {
        let isRegistered = BGTaskScheduler.shared.register(
            forTaskWithIdentifier: ApplicationConfiguration.addressCacheUpdateTaskIdentifier,
            using: nil
        ) { task in
            let handle = AddressCache.Tracker.shared.updateEndpoints { completion in
                self.scheduleAddressCacheUpdateTask()

                task.setTaskCompleted(success: completion.isSuccess)
            }

            task.expirationHandler = {
                handle.cancel()
            }
        }

        if isRegistered {
            logger.debug("Registered address cache update task.")
        } else {
            logger.error("Failed to register address cache update task.")
        }
    }

    func scheduleBackgroundTasks() {
        scheduleAppRefreshTask()
        scheduleKeyRotationTask()
        scheduleAddressCacheUpdateTask()
    }

    private func scheduleAppRefreshTask() {
        do {
            let date = RelayCache.Tracker.shared.getNextUpdateDate()

            let request = BGAppRefreshTaskRequest(
                identifier: ApplicationConfiguration.appRefreshTaskIdentifier
            )
            request.earliestBeginDate = date

            logger.debug("Schedule app refresh task at \(date.logFormatDate()).")

            try BGTaskScheduler.shared.submit(request)
        } catch {
            logger.error(
                error: error,
                message: "Could not schedule app refresh task."
            )
        }
    }

    private func scheduleKeyRotationTask() {
        do {
            guard let date = TunnelManager.shared.getNextKeyRotationDate() else {
                return
            }

            let request = BGProcessingTaskRequest(
                identifier: ApplicationConfiguration.privateKeyRotationTaskIdentifier
            )
            request.requiresNetworkConnectivity = true
            request.earliestBeginDate = date

            logger.debug("Schedule key rotation task at \(date.logFormatDate()).")

            try BGTaskScheduler.shared.submit(request)
        } catch {
            logger.error(
                error: error,
                message: "Could not schedule private key rotation task."
            )
        }
    }

    private func scheduleAddressCacheUpdateTask() {
        do {
            let date = AddressCache.Tracker.shared.nextScheduleDate()

            let request = BGProcessingTaskRequest(
                identifier: ApplicationConfiguration.addressCacheUpdateTaskIdentifier
            )
            request.requiresNetworkConnectivity = true
            request.earliestBeginDate = date

            logger.debug("Schedule address cache update task at \(date.logFormatDate()).")

            try BGTaskScheduler.shared.submit(request)
        } catch {
            logger.error(
                error: error,
                message: "Could not schedule address cache update task."
            )
        }
    }

    // MARK: - Private

    private func setupPaymentHandler() {
        AppStorePaymentManager.shared.delegate = self
        AppStorePaymentManager.shared.addPaymentObserver(TunnelManager.shared)
    }

    private func setupNotificationHandler() {
        NotificationManager.shared.notificationProviders = [
            AccountExpiryNotificationProvider(),
            TunnelStatusNotificationProvider(),
        ]
        UNUserNotificationCenter.current().delegate = self
    }
}

// MARK: - AppStorePaymentManagerDelegate

extension AppDelegate: AppStorePaymentManagerDelegate {
    func appStorePaymentManager(
        _ manager: AppStorePaymentManager,
        didRequestAccountTokenFor payment: SKPayment
    ) -> String? {
        // Since we do not persist the relation between payment and account number between the
        // app launches, we assume that all successful purchases belong to the active account
        // number.
        return TunnelManager.shared.deviceState.accountData?.number
    }
}

// MARK: - UNUserNotificationCenterDelegate

extension AppDelegate: UNUserNotificationCenterDelegate {
    func userNotificationCenter(
        _ center: UNUserNotificationCenter,
        didReceive response: UNNotificationResponse,
        withCompletionHandler completionHandler: @escaping () -> Void
    ) {
        let blockOperation = AsyncBlockOperation(dispatchQueue: .main) {
            if response.notification.request.identifier == accountExpiryNotificationIdentifier,
               response.actionIdentifier == UNNotificationDefaultActionIdentifier
            {
                let sceneDelegate = UIApplication.shared.connectedScenes
                    .first?.delegate as? SceneDelegate

                sceneDelegate?.showUserAccount()
            }

            completionHandler()
        }

        operationQueue.addOperation(blockOperation)
    }

    func userNotificationCenter(
        _ center: UNUserNotificationCenter,
        willPresent notification: UNNotification,
        withCompletionHandler completionHandler: @escaping (UNNotificationPresentationOptions)
            -> Void
    ) {
        if #available(iOS 14.0, *) {
            completionHandler([.list])
        } else {
            completionHandler([])
        }
    }
}
