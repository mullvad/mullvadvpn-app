//
//  AppDelegate.swift
//  MullvadVPN
//
//  Created by pronebird on 19/03/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import BackgroundTasks
import MullvadLogging
import MullvadREST
import MullvadSettings
import MullvadTypes
import Operations
import StoreKit
import UIKit
import UserNotifications

@UIApplicationMain
class AppDelegate: UIResponder, UIApplicationDelegate, UNUserNotificationCenterDelegate, StorePaymentManagerDelegate {
    private var logger: Logger!

    #if targetEnvironment(simulator)
    private var simulatorTunnelProviderHost: SimulatorTunnelProviderHost?
    #endif

    private let operationQueue = AsyncOperationQueue.makeSerial()

    private(set) var tunnelStore: TunnelStore!
    private(set) var tunnelManager: TunnelManager!
    private(set) var addressCache: REST.AddressCache!

    private var proxyFactory: REST.ProxyFactory!
    private(set) var apiProxy: APIQuerying!
    private(set) var accountsProxy: RESTAccountHandling!
    private(set) var devicesProxy: DeviceHandling!

    private(set) var addressCacheTracker: AddressCacheTracker!
    private(set) var relayCacheTracker: RelayCacheTracker!
    private(set) var storePaymentManager: StorePaymentManager!
    private var transportMonitor: TransportMonitor!
    private var relayConstraintsObserver: TunnelBlockObserver!
    private let migrationManager = MigrationManager()

    private(set) var accessMethodRepository = AccessMethodRepository()
    private(set) var shadowsocksLoader: ShadowsocksLoaderProtocol!
    private(set) var configuredTransportProvider: ProxyConfigurationTransportProvider!

    // MARK: - Application lifecycle

    // swiftlint:disable:next function_body_length
    func application(
        _ application: UIApplication,
        didFinishLaunchingWithOptions launchOptions: [UIApplication.LaunchOptionsKey: Any]?
    ) -> Bool {
        if ProcessInfo().arguments.contains("DisableAnimations") {
            UIView.setAnimationsEnabled(false)
        }

        let containerURL = ApplicationConfiguration.containerURL

        configureLogging()

        addressCache = REST.AddressCache(canWriteToCache: true, cacheDirectory: containerURL)
        addressCache.loadFromFile()

        setUpProxies(containerURL: containerURL)

        let ipOverrideWrapper = IPOverrideWrapper(
            relayCache: RelayCache(cacheDirectory: containerURL),
            ipOverrideRepository: IPOverrideRepository()
        )

        relayCacheTracker = RelayCacheTracker(
            relayCache: ipOverrideWrapper,
            application: application,
            apiProxy: apiProxy
        )

        addressCacheTracker = AddressCacheTracker(application: application, apiProxy: apiProxy, store: addressCache)

        tunnelStore = TunnelStore(application: application)
        tunnelManager = createTunnelManager(application: application)

        let constraintsUpdater = RelayConstraintsUpdater()
        relayConstraintsObserver = TunnelBlockObserver(didUpdateTunnelSettings: { _, settings in
            constraintsUpdater.onNewConstraints?(settings.relayConstraints)
        })
        tunnelManager.addObserver(relayConstraintsObserver)

        storePaymentManager = StorePaymentManager(
            backgroundTaskProvider: application,
            queue: .default(),
            apiProxy: apiProxy,
            accountsProxy: accountsProxy,
            transactionLog: .default
        )

        let urlSessionTransport = URLSessionTransport(urlSession: REST.makeURLSession())
        let shadowsocksCache = ShadowsocksConfigurationCache(cacheDirectory: containerURL)

        shadowsocksLoader = ShadowsocksLoader(
            shadowsocksCache: shadowsocksCache,
            relayCache: ipOverrideWrapper,
            constraintsUpdater: constraintsUpdater
        )

        configuredTransportProvider = ProxyConfigurationTransportProvider(
            shadowsocksLoader: shadowsocksLoader,
            addressCache: addressCache
        )

        let transportStrategy = TransportStrategy(
            datasource: accessMethodRepository,
            shadowsocksLoader: shadowsocksLoader
        )

        let transportProvider = TransportProvider(
            urlSessionTransport: urlSessionTransport,
            addressCache: addressCache,
            transportStrategy: transportStrategy
        )
        setUpTransportMonitor(transportProvider: transportProvider)
        setUpSimulatorHost(transportProvider: transportProvider)

        registerBackgroundTasks()
        setupPaymentHandler()
        setupNotifications()
        addApplicationNotifications(application: application)

        startInitialization(application: application)

        return true
    }

    private func createTunnelManager(application: UIApplication) -> TunnelManager {
        return TunnelManager(
            application: application,
            tunnelStore: tunnelStore,
            relayCacheTracker: relayCacheTracker,
            accountsProxy: accountsProxy,
            devicesProxy: devicesProxy,
            apiProxy: apiProxy,
            accessTokenManager: proxyFactory.configuration.accessTokenManager
        )
    }

    private func setUpProxies(containerURL: URL) {
        proxyFactory = REST.ProxyFactory.makeProxyFactory(
            transportProvider: REST.AnyTransportProvider { [weak self] in
                return self?.transportMonitor.makeTransport()
            },
            addressCache: addressCache
        )

        apiProxy = proxyFactory.createAPIProxy()
        accountsProxy = proxyFactory.createAccountsProxy()
        devicesProxy = proxyFactory.createDevicesProxy()
    }

    private func setUpTransportMonitor(transportProvider: TransportProvider) {
        transportMonitor = TransportMonitor(
            tunnelManager: tunnelManager,
            tunnelStore: tunnelStore,
            transportProvider: transportProvider
        )
    }

    private func setUpSimulatorHost(transportProvider: TransportProvider) {
        #if targetEnvironment(simulator)
        // Configure mock tunnel provider on simulator
        simulatorTunnelProviderHost = SimulatorTunnelProviderHost(
            relayCacheTracker: relayCacheTracker,
            transportProvider: transportProvider
        )
        SimulatorTunnelProvider.shared.delegate = simulatorTunnelProviderHost
        #endif
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
            forTaskWithIdentifier: BackgroundTask.appRefresh.identifier,
            using: nil
        ) { [self] task in
            let handle = relayCacheTracker.updateRelays { result in
                task.setTaskCompleted(success: result.isSuccess)
            }

            task.expirationHandler = {
                handle.cancel()
            }

            scheduleAppRefreshTask()
        }

        if isRegistered {
            logger.debug("Registered app refresh task.")
        } else {
            logger.error("Failed to register app refresh task.")
        }
    }

    private func registerKeyRotationTask() {
        let isRegistered = BGTaskScheduler.shared.register(
            forTaskWithIdentifier: BackgroundTask.privateKeyRotation.identifier,
            using: nil
        ) { [self] task in
            let handle = tunnelManager.rotatePrivateKey { [self] error in
                scheduleKeyRotationTask()

                task.setTaskCompleted(success: error == nil)
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
            forTaskWithIdentifier: BackgroundTask.addressCacheUpdate.identifier,
            using: nil
        ) { [self] task in
            let handle = addressCacheTracker.updateEndpoints { [self] result in
                scheduleAddressCacheUpdateTask()

                task.setTaskCompleted(success: result.isSuccess)
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

            let request = BGAppRefreshTaskRequest(identifier: BackgroundTask.appRefresh.identifier)
            request.earliestBeginDate = date

            logger.debug("Schedule app refresh task at \(date.logFormatDate()).")

            try BGTaskScheduler.shared.submit(request)
        } catch {
            logger.error(error: error, message: "Could not schedule app refresh task.")
        }
    }

    private func scheduleKeyRotationTask() {
        do {
            guard let date = tunnelManager.getNextKeyRotationDate() else {
                return
            }

            let request = BGProcessingTaskRequest(identifier: BackgroundTask.privateKeyRotation.identifier)
            request.requiresNetworkConnectivity = true
            request.earliestBeginDate = date

            logger.debug("Schedule key rotation task at \(date.logFormatDate()).")

            try BGTaskScheduler.shared.submit(request)
        } catch {
            logger.error(error: error, message: "Could not schedule private key rotation task.")
        }
    }

    private func scheduleAddressCacheUpdateTask() {
        do {
            let date = addressCacheTracker.nextScheduleDate()

            let request = BGProcessingTaskRequest(identifier: BackgroundTask.addressCacheUpdate.identifier)
            request.requiresNetworkConnectivity = true
            request.earliestBeginDate = date

            logger.debug("Schedule address cache update task at \(date.logFormatDate()).")

            try BGTaskScheduler.shared.submit(request)
        } catch {
            logger.error(error: error, message: "Could not schedule address cache update task.")
        }
    }

    // MARK: - Private

    private func configureLogging() {
        var loggerBuilder = LoggerBuilder()
        loggerBuilder.addFileOutput(fileURL: ApplicationConfiguration.logFileURL(for: .mainApp))
        #if DEBUG
        loggerBuilder.addOSLogOutput(subsystem: ApplicationTarget.mainApp.bundleIdentifier)
        #endif
        loggerBuilder.install()

        logger = Logger(label: "AppDelegate")
    }

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

    private func setupNotifications() {
        NotificationManager.shared.notificationProviders = [
            TunnelStatusNotificationProvider(tunnelManager: tunnelManager),
            AccountExpirySystemNotificationProvider(tunnelManager: tunnelManager),
            AccountExpiryInAppNotificationProvider(tunnelManager: tunnelManager),
            RegisteredDeviceInAppNotificationProvider(tunnelManager: tunnelManager),
        ]
        UNUserNotificationCenter.current().delegate = self
    }

    private func startInitialization(application: UIApplication) {
        let wipeSettingsOperation = getWipeSettingsOperation()
        let loadTunnelStoreOperation = getLoadTunnelStoreOperation()
        let migrateSettingsOperation = getMigrateSettingsOperation(application: application)
        let initTunnelManagerOperation = getInitTunnelManagerOperation()

        migrateSettingsOperation.addDependencies([wipeSettingsOperation, loadTunnelStoreOperation])
        initTunnelManagerOperation.addDependency(migrateSettingsOperation)

        operationQueue.addOperations(
            [
                wipeSettingsOperation,
                loadTunnelStoreOperation,
                migrateSettingsOperation,
                initTunnelManagerOperation,
            ],
            waitUntilFinished: false
        )
    }

    private func getLoadTunnelStoreOperation() -> AsyncBlockOperation {
        AsyncBlockOperation(dispatchQueue: .main) { [self] finish in
            tunnelStore.loadPersistentTunnels { [self] error in
                if let error {
                    logger.error(
                        error: error,
                        message: "Failed to load persistent tunnels."
                    )
                }
                finish(nil)
            }
        }
    }

    private func getMigrateSettingsOperation(application: UIApplication) -> AsyncBlockOperation {
        AsyncBlockOperation(dispatchQueue: .main) { [self] finish in
            migrationManager
                .migrateSettings(store: SettingsManager.store) { [self] migrationResult in
                    switch migrationResult {
                    case .success:
                        // Tell the tunnel to re-read tunnel configuration after migration.
                        logger.debug("Reconnect the tunnel after settings migration.")
                        tunnelManager.reconnectTunnel(selectNewRelay: true)
                        fallthrough

                    case .nothing:
                        finish(nil)

                    case let .failure(error):
                        let migrationUIHandler = application.connectedScenes
                            .first { $0 is SettingsMigrationUIHandler } as? SettingsMigrationUIHandler

                        if let migrationUIHandler {
                            migrationUIHandler.showMigrationError(error) {
                                finish(error)
                            }
                        } else {
                            finish(error)
                        }
                    }
                }
        }
    }

    private func getInitTunnelManagerOperation() -> AsyncBlockOperation {
        // This operation is always treated as successful no matter what the configuration load yields.
        // If the tunnel settings or device state can't be read, we simply pretend they are not there
        // and leave user in logged out state. VPN config will be removed as well.
        AsyncBlockOperation(dispatchQueue: .main) { finish in
            self.tunnelManager.loadConfiguration {
                self.logger.debug("Finished initialization.")

                NotificationManager.shared.updateNotifications()
                self.storePaymentManager.start()

                finish(nil)
            }
        }
    }

    /// Returns an operation that acts on two conditions:
    /// 1. Has the app been launched at least once after install? (`FirstTimeLaunch.hasFinished`)
    /// 2. Has the app - at some point in time - been updated from a version compatible with wiping settings?
    /// (`SettingsManager.getShouldWipeSettings()`)
    /// If (1) is `false` and (2) is `true`, we know that the app has been freshly installed/reinstalled and is
    /// compatible, thus triggering a settings wipe.
    private func getWipeSettingsOperation() -> AsyncBlockOperation {
        AsyncBlockOperation {
            if !FirstTimeLaunch.hasFinished, SettingsManager.getShouldWipeSettings() {
                if let deviceState = try? SettingsManager.readDeviceState(),
                   let accountData = deviceState.accountData,
                   let deviceData = deviceState.deviceData {
                    _ = self.devicesProxy.deleteDevice(
                        accountNumber: accountData.number,
                        identifier: deviceData.identifier,
                        retryStrategy: .noRetry
                    ) { _ in
                        // Do nothing.
                    }
                }

                SettingsManager.resetStore(completely: true)
                self.accessMethodRepository.reloadWithDefaultsAfterDataRemoval()
            }

            FirstTimeLaunch.setHasFinished()
            SettingsManager.setShouldWipeSettings()
        }
    }

    // MARK: - StorePaymentManagerDelegate

    func storePaymentManager(_ manager: StorePaymentManager, didRequestAccountTokenFor payment: SKPayment) -> String? {
        // Since we do not persist the relation between payment and account number between the
        // app launches, we assume that all successful purchases belong to the active account
        // number.
        tunnelManager.deviceState.accountData?.number
    }

    // MARK: - UNUserNotificationCenterDelegate

    func userNotificationCenter(
        _ center: UNUserNotificationCenter,
        didReceive response: UNNotificationResponse,
        withCompletionHandler completionHandler: @escaping () -> Void
    ) {
        let blockOperation = AsyncBlockOperation(dispatchQueue: .main) {
            NotificationManager.shared.handleSystemNotificationResponse(response)

            completionHandler()
        }

        operationQueue.addOperation(blockOperation)
    }

    func userNotificationCenter(
        _ center: UNUserNotificationCenter,
        willPresent notification: UNNotification,
        withCompletionHandler completionHandler: @escaping (UNNotificationPresentationOptions) -> Void
    ) {
        completionHandler([.list, .banner, .sound])
    }

    // swiftlint:disable:next file_length
}
