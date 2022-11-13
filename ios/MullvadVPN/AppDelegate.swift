//
//  AppDelegate.swift
//  MullvadVPN
//
//  Created by pronebird on 19/03/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import BackgroundTasks
import Intents
import MullvadLogging
import MullvadREST
import Operations
import RelayCache
import StoreKit
import UIKit
import UserNotifications

@UIApplicationMain
class AppDelegate: UIResponder, UIApplicationDelegate, UNUserNotificationCenterDelegate,
    StorePaymentManagerDelegate
{
    private var logger: Logger!

    #if targetEnvironment(simulator)
    private var simulatorTunnelProviderHost: SimulatorTunnelProviderHost?
    #endif

    private let operationQueue: AsyncOperationQueue = {
        let operationQueue = AsyncOperationQueue()
        operationQueue.maxConcurrentOperationCount = 1
        return operationQueue
    }()

    private(set) var tunnelManager: TunnelManager!
    private(set) var addressCache: REST.AddressCache!

    private var proxyFactory: REST.ProxyFactory!
    private(set) var apiProxy: REST.APIProxy!
    private(set) var accountsProxy: REST.AccountsProxy!
    private(set) var devicesProxy: REST.DevicesProxy!

    private(set) var addressCacheTracker: AddressCacheTracker!
    private(set) var relayCacheTracker: RelayCacheTracker!
    private(set) var storePaymentManager: StorePaymentManager!
    private var transportMonitor: TransportMonitor!

    // MARK: - Application lifecycle

    func application(
        _ application: UIApplication,
        didFinishLaunchingWithOptions launchOptions: [UIApplication.LaunchOptionsKey: Any]?
    ) -> Bool {
        initLoggingSystem(
            bundleIdentifier: Bundle.main.bundleIdentifier!,
            applicationGroupIdentifier: ApplicationConfiguration.securityGroupIdentifier
        )

        logger = Logger(label: "AppDelegate")

        addressCache = REST.AddressCache(
            securityGroupIdentifier: ApplicationConfiguration.securityGroupIdentifier,
            isReadOnly: false
        )!

        proxyFactory = REST.ProxyFactory.makeProxyFactory(
            transportProvider: { [weak self] in
                return self?.transportMonitor.transport
            },
            addressCache: addressCache
        )

        apiProxy = proxyFactory.createAPIProxy()
        accountsProxy = proxyFactory.createAccountsProxy()
        devicesProxy = proxyFactory.createDevicesProxy()

        relayCacheTracker = RelayCacheTracker(application: application, apiProxy: apiProxy)
        addressCacheTracker = AddressCacheTracker(
            application: application,
            apiProxy: apiProxy,
            store: addressCache
        )

        tunnelManager = TunnelManager(
            application: application,
            relayCacheTracker: relayCacheTracker,
            accountsProxy: accountsProxy,
            devicesProxy: devicesProxy
        )

        storePaymentManager = StorePaymentManager(
            application: application,
            queue: .default(),
            apiProxy: apiProxy,
            accountsProxy: accountsProxy
        )

        transportMonitor = TransportMonitor(tunnelManager: tunnelManager)

        #if targetEnvironment(simulator)
        // Configure mock tunnel provider on simulator
        simulatorTunnelProviderHost = SimulatorTunnelProviderHost(
            relayCacheTracker: relayCacheTracker
        )
        SimulatorTunnelProvider.shared.delegate = simulatorTunnelProviderHost
        #endif

        registerBackgroundTasks()
        setupPaymentHandler()
        setupNotificationHandler()
        addApplicationNotifications(application: application)

        let setupTunnelManagerOperation =
            AsyncBlockOperation(dispatchQueue: .main) { operation in
                SettingsManager.migrateStore(with: self.proxyFactory) { error in
                    precondition(error == nil)

                    self.tunnelManager.loadConfiguration { error in
                        // TODO: avoid throwing fatal error and show the problem report UI instead.
                        if let error = error {
                            fatalError(error.localizedDescription)
                        }

                        self.logger.debug("Finished initialization.")

                        NotificationManager.shared.updateNotifications()
                        self.storePaymentManager.startPaymentQueueMonitoring()

                        operation.finish()
                    }
                }
            }

        operationQueue.addOperation(setupTunnelManagerOperation)

        return true
    }

    func application(_ application: UIApplication, handlerFor intent: INIntent) -> Any? {
        switch intent {
        case is StartVPNIntent:
            return StartVPNIntentHandler(tunnelManager: tunnelManager)
        case is StopVPNIntent:
            return StopVPNIntentHandler(tunnelManager: tunnelManager)
        case is ReconnectVPNIntent:
            return ReconnectVPNIntentHandler(tunnelManager: tunnelManager)
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
    ) {}

    // MARK: - Notifications

    @objc private func didBecomeActive(_ notification: Notification) {
        tunnelManager.refreshTunnelStatus()
        tunnelManager.startPeriodicPrivateKeyRotation()
        relayCacheTracker.startPeriodicUpdates()
        addressCacheTracker.startPeriodicUpdates()
    }

    @objc private func willResignActive(_ notification: Notification) {
        tunnelManager.stopPeriodicPrivateKeyRotation()
        relayCacheTracker.stopPeriodicUpdates()
        addressCacheTracker.stopPeriodicUpdates()
    }

    @objc private func didEnterBackground(_ notification: Notification) {
        scheduleBackgroundTasks()
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
            let handle = self.relayCacheTracker.updateRelays { completion in
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
            let handle = self.tunnelManager.rotatePrivateKey(forceRotate: false) { completion in
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
            let handle = self.addressCacheTracker.updateEndpoints { completion in
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

    private func scheduleBackgroundTasks() {
        scheduleAppRefreshTask()
        scheduleKeyRotationTask()
        scheduleAddressCacheUpdateTask()
    }

    private func scheduleAppRefreshTask() {
        do {
            let date = relayCacheTracker.getNextUpdateDate()

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
            guard let date = tunnelManager.getNextKeyRotationDate() else {
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
            let date = addressCacheTracker.nextScheduleDate()

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

    private func addApplicationNotifications(application: UIApplication) {
        let notificationCenter = NotificationCenter.default

        notificationCenter.addObserver(
            self,
            selector: #selector(didBecomeActive(_:)),
            name: UIApplication.didBecomeActiveNotification,
            object: application
        )
        notificationCenter.addObserver(
            self,
            selector: #selector(willResignActive(_:)),
            name: UIApplication.willResignActiveNotification,
            object: application
        )
        notificationCenter.addObserver(
            self,
            selector: #selector(didEnterBackground(_:)),
            name: UIApplication.didEnterBackgroundNotification,
            object: application
        )
    }

    private func setupPaymentHandler() {
        storePaymentManager.delegate = self
        storePaymentManager.addPaymentObserver(tunnelManager)
    }

    private func setupNotificationHandler() {
        NotificationManager.shared.notificationProviders = [
            AccountExpiryNotificationProvider(tunnelManager: tunnelManager),
            TunnelStatusNotificationProvider(tunnelManager: tunnelManager),
        ]
        UNUserNotificationCenter.current().delegate = self
    }

    // MARK: - StorePaymentManagerDelegate

    func storePaymentManager(
        _ manager: StorePaymentManager,
        didRequestAccountTokenFor payment: SKPayment
    ) -> String? {
        // Since we do not persist the relation between payment and account number between the
        // app launches, we assume that all successful purchases belong to the active account
        // number.
        return tunnelManager.deviceState.accountData?.number
    }

    // MARK: - UNUserNotificationCenterDelegate

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
