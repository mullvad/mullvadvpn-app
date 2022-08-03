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

    // An instance of scene delegate used on iOS 12 or earlier.
    private var sceneDelegate: SceneDelegate?

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

        if #available(iOS 13.0, *) {
            registerBackgroundTasks()
        } else {
            application.setMinimumBackgroundFetchInterval(
                ApplicationConfiguration.minimumBackgroundFetchInterval
            )
        }

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

        if #available(iOS 13, *) {
            // no-op
        } else {
            sceneDelegate = SceneDelegate()
            sceneDelegate?.setupScene(windowFactory: ClassicWindowFactory())
        }

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

    func application(
        _ application: UIApplication,
        performFetchWithCompletionHandler completionHandler: @escaping (UIBackgroundFetchResult)
            -> Void
    ) {
        logger.debug("Start background refresh.")

        let updateAddressCacheOperation = ResultBlockOperation<Bool, Error> { operation in
            let handle = AddressCache.Tracker.shared.updateEndpoints { completion in
                operation.finish(completion: completion)
            }

            operation.addCancellationBlock {
                handle.cancel()
            }
        }

        let updateRelaysOperation = ResultBlockOperation<RelayCache.FetchResult, Error>
        { operation in
            let handle = RelayCache.Tracker.shared.updateRelays { completion in
                operation.finish(completion: completion)
            }

            operation.addCancellationBlock {
                handle.cancel()
            }
        }

        let rotatePrivateKeyOperation = ResultBlockOperation<Bool, Error> { operation in
            let handle = TunnelManager.shared.rotatePrivateKey(forceRotate: false) { completion in
                operation.finish(completion: completion)
            }

            operation.addCancellationBlock {
                handle.cancel()
            }
        }
        rotatePrivateKeyOperation.addDependencies([
            updateRelaysOperation,
            updateAddressCacheOperation,
        ])

        let operations = [
            updateAddressCacheOperation,
            updateRelaysOperation,
            rotatePrivateKeyOperation,
        ]

        let completeOperation = TransformOperation<UIBackgroundFetchResult, Void, Never>(
            dispatchQueue: .main
        )

        completeOperation.setExecutionBlock { backgroundFetchResult in
            self.logger.debug("Finish background refresh. Status: \(backgroundFetchResult).")

            completionHandler(backgroundFetchResult)
        }

        completeOperation.injectMany(context: [UIBackgroundFetchResult]())
            .injectCompletion(from: updateAddressCacheOperation, via: { results, completion in
                results.append(completion.backgroundFetchResult { $0 })
            })
            .injectCompletion(from: updateRelaysOperation, via: { results, completion in
                results.append(completion.backgroundFetchResult { $0 == .newContent })
            })
            .injectCompletion(from: rotatePrivateKeyOperation, via: { results, completion in
                results.append(completion.backgroundFetchResult { $0 })
            })
            .reduce { operationResults in
                let initialResult = operationResults.first ?? .failed
                let backgroundFetchResult = operationResults
                    .reduce(initialResult) { partialResult, other in
                        return partialResult.combine(with: other)
                    }

                return backgroundFetchResult
            }

        let groupOperation = GroupOperation(operations: operations)
        groupOperation.addObserver(
            BackgroundObserver(name: "Background refresh", cancelUponExpiration: true)
        )

        let operationQueue = AsyncOperationQueue()
        operationQueue.addOperation(groupOperation)
        operationQueue.addOperation(completeOperation)
    }

    // MARK: - UISceneSession lifecycle

    @available(iOS 13.0, *)
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

    @available(iOS 13.0, *)
    func application(
        _ application: UIApplication,
        didDiscardSceneSessions sceneSessions: Set<UISceneSession>
    ) {
        // Called when the user discards a scene session.
        // If any sessions were discarded while the application was not running, this will be called shortly after application:didFinishLaunchingWithOptions.
        // Use this method to release any resources that were specific to the discarded scenes, as they will not return.
    }

    // MARK: - Background tasks

    @available(iOS 13, *)
    private func registerBackgroundTasks() {
        registerAppRefreshTask()
        registerAddressCacheUpdateTask()
        registerKeyRotationTask()
    }

    @available(iOS 13.0, *)
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

    @available(iOS 13.0, *)
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

    @available(iOS 13.0, *)
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

    @available(iOS 13.0, *)
    func scheduleBackgroundTasks() {
        scheduleAppRefreshTask()
        scheduleKeyRotationTask()
        scheduleAddressCacheUpdateTask()
    }

    @available(iOS 13.0, *)
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
                chainedError: AnyChainedError(error),
                message: "Could not schedule app refresh task."
            )
        }
    }

    @available(iOS 13.0, *)
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
                chainedError: AnyChainedError(error),
                message: "Could not schedule private key rotation task."
            )
        }
    }

    @available(iOS 13.0, *)
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
                chainedError: AnyChainedError(error),
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
            TunnelErrorNotificationProvider(),
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
                if #available(iOS 13.0, *) {
                    let sceneDelegate = UIApplication.shared.connectedScenes
                        .first?.delegate as? SceneDelegate

                    sceneDelegate?.showUserAccount()
                } else {
                    self.sceneDelegate?.showUserAccount()
                }
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
