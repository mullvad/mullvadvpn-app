// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import BackgroundTasks
import MullvadLogging
import MullvadMockData
import MullvadREST
import MullvadRustRuntime
import MullvadSettings
import MullvadTypes
import Operations
import UIKit
import UserNotifications

@main
class AppDelegate: UIResponder, UIApplicationDelegate, UNUserNotificationCenterDelegate, @unchecked Sendable {
    nonisolated(unsafe) private var logger: Logger!

    #if targetEnvironment(simulator)
        private var simulatorTunnelProviderHost: SimulatorTunnelProviderHost?
    #endif

    private let operationQueue = AsyncOperationQueue.makeSerial()

    private(set) var tunnelStore: TunnelStore!
    nonisolated(unsafe) private(set) var tunnelManager: TunnelManager!

    private var proxyFactory: ProxyFactoryProtocol!
    private(set) var apiProxy: APIQuerying!
    private(set) var accountsProxy: RESTAccountHandling!
    nonisolated(unsafe) private(set) var devicesProxy: DeviceHandling!

    private(set) var addressCacheUpdateScheduler: AddressCacheUpdateScheduler!
    nonisolated(unsafe) private(set) var relayCacheTracker: RelayCacheTracker!
    nonisolated(unsafe) private(set) var storePaymentManager: StorePaymentManager!
    nonisolated(unsafe) private var apiTransportMonitor: APITransportMonitor!
    private var settingsObserver: TunnelBlockObserver!
    private var migrationManager: MigrationManager!

    let settingsManager = SettingsManager()
    nonisolated(unsafe) private(set) var accessMethodRepository: AccessMethodRepository!

    nonisolated(unsafe) private(set) var appPreferences = AppPreferences()
    private(set) var shadowsocksLoader: ShadowsocksLoader!
    private(set) lazy var ipOverrideRepository = IPOverrideRepository(settingsStore: settingsManager.store)
    private(set) var relaySelector: RelaySelectorWrapper!
    private(set) var launchArguments = LaunchArguments()
    var apiContext: MullvadApiContext!
    var accessMethodReceiver: MullvadAccessMethodReceiver!
    private var shadowsocksCacheCleaner: ShadowsocksCacheCleaner!
    let breadcrumbsProvider = BreadcrumbsProvider()
    let inAppLogObserver = InAppLogBlockObserver()

    let notificationSettingsListener = NotificationSettingsListener()
    private var notificationSettingsUpdater: NotificationSettingsUpdater!

    let migratedSettingsListener = MigratedSettingsListener()
    private var migratedSettingsUpdater: MigratedSettingsUpdater!

    nonisolated(unsafe) private(set) var logRedactor: LogRedacting!
    private let containerURL = ApplicationConfiguration.containerURL

    // MARK: - Application lifecycle

    func application(
        _ application: UIApplication,
        didFinishLaunchingWithOptions launchOptions: [UIApplication.LaunchOptionsKey: Any]?
    ) -> Bool {
        application.accessibilityLanguage = Locale.current.language.languageCode?.identifier

        if let overriddenLaunchArguments = try? ProcessInfo.processInfo.decode(LaunchArguments.self) {
            launchArguments = overriddenLaunchArguments
        }

        migrationManager = MigrationManager(cacheDirectory: containerURL, settingsManager: settingsManager)
        configureLogging()

        // Can be removed in 2027.1, see `FileCacheMaintenance`.
        Task.detached(priority: .utility) { [containerURL, logger] in
            let removedOrphanCount = FileCacheMaintenance.removeStaleCacheFiles(in: containerURL)
            if removedOrphanCount > 0 {
                logger?.warning("Removed \(removedOrphanCount) orphaned file cache temporary file(s).")
            }
        }

        let ipOverrideWrapper = IPOverrideWrapper(
            relayCache: RelayCache(cacheDirectory: containerURL),
            ipOverrideRepository: ipOverrideRepository
        )

        let tunnelSettingsListener = TunnelSettingsListener()
        let tunnelSettingsUpdater = SettingsUpdater(listener: tunnelSettingsListener)

        notificationSettingsUpdater = NotificationSettingsUpdater(listener: notificationSettingsListener)
        migratedSettingsUpdater = MigratedSettingsUpdater(listener: migratedSettingsListener)

        let shadowsocksCache = ShadowsocksConfigurationCache(cacheDirectory: containerURL)
        let shadowsocksRelaySelector = ShadowsocksRelaySelector(
            relayCache: ipOverrideWrapper
        )

        let tunnelSettings = (try? settingsManager.readSettings()) ?? LatestTunnelSettings()

        shadowsocksLoader = ShadowsocksLoader(
            cache: shadowsocksCache,
            relaySelector: shadowsocksRelaySelector,
            tunnelSettings: tunnelSettings,
            settingsUpdater: tunnelSettingsUpdater
        )

        shadowsocksCacheCleaner = ShadowsocksCacheCleaner(cache: shadowsocksCache)

        accessMethodRepository = AccessMethodRepository(
            shadowsocksCiphers: ShadowsocksCipherService().getCiphers(),
            settingsStore: settingsManager.store
        )
        let opaqueAccessMethodSettingsWrapper = initAccessMethodSettingsWrapper(
            methods: accessMethodRepository.fetchAll()
        )

        // swift-format-ignore: NeverUseForceTry
        apiContext = try! MullvadApiContext(
            host: REST.defaultAPIHostname,
            address: REST.defaultAPIEndpoint.description,
            domain: REST.encryptedDNSHostname,
            shadowsocksProvider: shadowsocksLoader,
            accessMethodWrapper: opaqueAccessMethodSettingsWrapper,
            accessMethodChangeListeners: [accessMethodRepository, shadowsocksCacheCleaner]
        )

        accessMethodReceiver = MullvadAccessMethodReceiver(
            apiContext: apiContext,
            validShadowsocksCiphers: accessMethodRepository.shadowsocksCiphers,
            accessMethodsDataSource: accessMethodRepository.accessMethodsPublisher,
            requestDataSource: accessMethodRepository.requestAccessMethodPublisher
        )

        setUpProxies(containerURL: containerURL)
        let backgroundTaskProvider = BackgroundTaskProvider(
            backgroundTimeRemaining: application.backgroundTimeRemaining,
            application: application
        )

        relayCacheTracker = RelayCacheTracker(
            relayCache: ipOverrideWrapper,
            backgroundTaskProvider: backgroundTaskProvider,
            apiProxy: apiProxy
        )

        addressCacheUpdateScheduler = AddressCacheUpdateScheduler(
            backgroundTaskProvider: backgroundTaskProvider,
            apiProxy: apiProxy,
            apiContext: apiContext
        )

        tunnelStore = TunnelStore(application: backgroundTaskProvider)

        relaySelector = RelaySelectorWrapper(
            relayCache: ipOverrideWrapper
        )
        tunnelManager = createTunnelManager(
            backgroundTaskProvider: backgroundTaskProvider,
            relaySelector: relaySelector
        )

        settingsObserver = TunnelBlockObserver(
            didLoadConfiguration: { [weak self] tunnelManager in
                guard let self else { return }
                // Redact legacy numbers that do not match the account number regex.
                logRedactor.addCustomString(tunnelManager.deviceState.accountData?.number ?? "")
            },
            didUpdateTunnelSettings: { _, settings in
                tunnelSettingsListener.onNewSettings?(settings)
            })
        tunnelManager.addObserver(settingsObserver)

        #if DEBUG
            tunnelManager.onInAppLogEntries = { [weak self] entries in
                DispatchQueue.main.async {
                    entries.forEach { self?.inAppLogObserver.didAddLogEntry($0) }
                }
            }
        #endif

        storePaymentManager = StorePaymentManager(
            interactor: StorePaymentManagerInteractor(
                tunnelManager: tunnelManager,
                apiProxy: apiProxy,
                accountProxy: accountsProxy
            )
        )

        let apiRequestFactory = MullvadApiRequestFactory(
            apiContext: apiContext,
            encoder: REST.Coding.makeJSONEncoder()
        )
        let apiTransportProvider = APITransportProvider(requestFactory: apiRequestFactory)

        apiTransportMonitor = APITransportMonitor(
            tunnelManager: tunnelManager,
            tunnelStore: tunnelStore,
            requestFactory: apiRequestFactory
        )

        setUpSimulatorHost(
            apiTransportProvider: apiTransportProvider,
            relaySelector: relaySelector
        )
        registerBackgroundTasks()
        setupNotifications(
            tunnelSettings: tunnelSettings,
            tunnelSettingsUpdater: tunnelSettingsUpdater
        )
        addApplicationNotifications(application: application)
        startInitialization(application: application)

        // Pre-warm @Observable infrastructure for LocationNode to avoid first-render lag
        // in SelectLocationView. SwiftUI's observation system has initialization overhead
        // that is cached after first use.
        DispatchQueue.global(qos: .userInitiated).async {
            _ = LocationNode(name: "", code: "")
        }

        return true
    }

    private func createTunnelManager(
        backgroundTaskProvider: BackgroundTaskProviding,
        relaySelector: RelaySelectorProtocol
    ) -> TunnelManager {
        return TunnelManager(
            backgroundTaskProvider: backgroundTaskProvider,
            tunnelStore: tunnelStore,
            relayCacheTracker: relayCacheTracker,
            accountsProxy: accountsProxy,
            devicesProxy: devicesProxy,
            apiProxy: apiProxy,
            relaySelector: relaySelector,
            settingsManager: settingsManager
        )
    }

    private func setUpProxies(containerURL: URL) {
        if launchArguments.target == .screenshots {
            proxyFactory = MockProxyFactory.makeProxyFactory(
                apiTransportProvider: REST.AnyAPITransportProvider { [weak self] in
                    self?.apiTransportMonitor.makeTransport()
                }
            )
        } else {
            proxyFactory = REST.ProxyFactory.makeProxyFactory(
                apiTransportProvider: REST.AnyAPITransportProvider { [weak self] in
                    self?.apiTransportMonitor.makeTransport()
                }
            )
        }
        apiProxy = proxyFactory.createAPIProxy()
        accountsProxy = proxyFactory.createAccountsProxy()
        devicesProxy = proxyFactory.createDevicesProxy()
    }

    private func setUpSimulatorHost(
        apiTransportProvider: APITransportProvider,
        relaySelector: RelaySelectorWrapper
    ) {
        #if targetEnvironment(simulator)
            // Configure mock tunnel provider on simulator
            simulatorTunnelProviderHost = SimulatorTunnelProviderHost(
                relaySelector: relaySelector,
                apiTransportProvider: apiTransportProvider,
                settingsManager: settingsManager
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
        addressCacheUpdateScheduler.startPeriodicUpdates()
    }

    @objc private func willResignActive(_ notification: Notification) {
        tunnelManager.stopPeriodicPrivateKeyRotation()
        relayCacheTracker.stopPeriodicUpdates()
        addressCacheUpdateScheduler.stopPeriodicUpdates()
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
            using: .main
        ) { [self] task in
            nonisolated(unsafe) let handle = relayCacheTracker.updateRelays { result in
                task.setTaskCompleted(success: result.isSuccess)
            }

            task.expirationHandler = { @Sendable in
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
            using: .main
        ) { [self] task in
            nonisolated(unsafe) let handle = tunnelManager.rotatePrivateKey { [self] error in
                scheduleKeyRotationTask()
                task.setTaskCompleted(success: error == nil)
            }

            task.expirationHandler = { @Sendable in
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
            using: .main
        ) { [self] task in
            addressCacheUpdateScheduler.updateEndpoints { [self] result in
                scheduleAddressCacheUpdateTask()
                task.setTaskCompleted(success: result.isSuccess)
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

            logger.debug("Schedule app refresh task at \(date.logFormatted).")

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

            logger.debug("Schedule key rotation task at \(date.logFormatted).")

            try BGTaskScheduler.shared.submit(request)
        } catch {
            logger.error(error: error, message: "Could not schedule private key rotation task.")
        }
    }

    private func scheduleAddressCacheUpdateTask() {
        do {
            let date = addressCacheUpdateScheduler.nextScheduleDate()

            let request = BGProcessingTaskRequest(identifier: BackgroundTask.addressCacheUpdate.identifier)
            request.requiresNetworkConnectivity = true
            request.earliestBeginDate = date

            logger.debug("Schedule address cache update task at \(date.logFormatted).")

            try BGTaskScheduler.shared.submit(request)
        } catch {
            logger.error(error: error, message: "Could not schedule address cache update task.")
        }
    }

    // MARK: - Private

    private func configureLogging() {
        let header = "MullvadVPN version \(Bundle.main.productVersion)"
        let loggerBuilder = LoggerBuilder.shared

        // Create the Rust-backed log redactor with container paths
        let redactor = RustLogRedactor(containerPaths: [ApplicationConfiguration.containerURL.path])
        self.logRedactor = redactor

        loggerBuilder.addFileOutput(
            fileURL: ApplicationConfiguration.newLogFileURL(for: .mainApp, in: ApplicationConfiguration.containerURL),
            header: header
        )
        #if DEBUG
            loggerBuilder.addOSLogOutput(subsystem: ApplicationTarget.mainApp.bundleIdentifier)
            loggerBuilder.addInAppLogOutput(
                process: .app,
                observer: inAppLogObserver
            )
        #endif
        loggerBuilder.install(redactor)

        // Initialize Rust logging to forward to Swift Logger
        RustLogging.initialize(logger: Logger(label: "Rust"))

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

    private func setupNotifications(
        tunnelSettings: LatestTunnelSettings,
        tunnelSettingsUpdater: SettingsUpdater

    ) {
        let appVersionService = AppVersionService(
            urlSession: URLSession.shared,
            appPreferences: appPreferences,
            mainAppBundleIdentifier: ApplicationTarget.mainApp.bundleIdentifier
        )

        NotificationManager.shared.notificationProviders = [
            SettingsMigrationInAppNotificationProvider(
                tunnelManager: tunnelManager,
                migratedSettingsUpdater: migratedSettingsUpdater),
            LatestChangesNotificationProvider(appPreferences: appPreferences),
            TunnelStatusNotificationProvider(tunnelManager: tunnelManager),
            AccountExpirySystemNotificationProvider(
                isNotificationEnabled: appPreferences.notificationSettings.isAccountNotificationEnabled,
                notificationSettingsUpdater: notificationSettingsUpdater, tunnelManager: tunnelManager),
            AccountExpiryInAppNotificationProvider(tunnelManager: tunnelManager),
            NewDeviceNotificationProvider(tunnelManager: tunnelManager),
            NewAppVersionInAppNotificationProvider(
                tunnelManager: tunnelManager,
                appVersionService: appVersionService
            ),
            InvalidShadowsocksCipherInAppNotificationProvider(breadcrumbsProvider: breadcrumbsProvider),
        ]
        UNUserNotificationCenter.current().delegate = self
    }

    // MARK: Initialisation tasks, which are (mostly) performed asynchronously

    private func startInitialization(application: UIApplication) {
        // the new structured code: things here run first, before we run the legacy operations.
        // in the fullness of time, this section will grow and the latter will shrink to nothing
        Task {
            doWipeSettingsIfNeeded()
            async let loadTunnelStore: () = doLoadTunnelStore()
            async let getDefaultLocation: () = doGetDefaultLocation()
            _ = await [loadTunnelStore, getDefaultLocation]
            await doMigrateSettings(application: application)
            await doResolveDeprecatedSettings()
            await doInitTunnelManager()
            await storePaymentManager.start()
        }
    }

    private func doLoadTunnelStore() async {
        do {
            try await tunnelStore.loadPersistentTunnels()
        } catch {
            logger.error(
                error: error,
                message: "Failed to load persistent tunnels."
            )
        }
    }

    private func doMigrateSettings(application: UIApplication) async {
        // this triggers an operation that needs to be carried out on the main queue, as
        // it uses NSFileCoordinator to maintain a lock shared between the app and extension
        // it consumes any error states produced in the process
        await withCheckedContinuation { continuation in
            DispatchQueue.main.async {
                self.migrationManager
                    .migrateSettings(store: self.settingsManager.store) { [self] migrationResult in
                        switch migrationResult {
                        case .success:
                            // Tell the tunnel to re-read tunnel configuration after migration.
                            logger.debug("Successful migration from UI Process")
                            tunnelManager.reconnectTunnel(selectNewRelay: true)
                            fallthrough

                        case .nothing:
                            logger.debug("Attempted migration from UI Process, but found nothing to do")
                            continuation.resume(returning: ())

                        case let .failure(error):
                            logger.error("Failed migration from UI Process: \(error)")
                            // SAFETY NOTE: We can assume this to be running on the same thread that
                            // `MigrationManager.migrateSettings` was called from, as the callback is
                            // called through `NSFileCoordinator`, which runs synchronously on one thread
                            // and is used for locking between processes using the filesystem.
                            MainActor.assumeIsolated {
                                let migrationUIHandler =
                                    application.connectedScenes
                                    .first { $0 is SettingsMigrationUIHandler } as? SettingsMigrationUIHandler

                                if let migrationUIHandler {
                                    migrationUIHandler.showMigrationError(error) {
                                        continuation.resume(returning: ())
                                    }
                                } else {
                                    continuation.resume(returning: ())
                                }
                            }
                        }
                    }
            }
        }
    }

    private func doInitTunnelManager() async {
        // This operation is always treated as successful no matter what the configuration load yields.
        // If the tunnel settings or device state can't be read, we simply pretend they are not there
        // and leave user in logged out state. VPN config will be removed as well.
        await withCheckedContinuation { continuation in
            self.tunnelManager.loadConfiguration {
                self.logger.debug("Finished initialization.")

                NotificationManager.shared.updateNotifications()

                continuation.resume(returning: ())
            }
        }
    }

    /// 1. If the app has never been launched, preload with default settings.
    /// - or -
    /// 1. Has the app been launched at least once after install? (`FirstTimeLaunch.hasFinished`)
    /// 2. Has the app - at some point in time - been updated from a version compatible with wiping settings?
    /// (`SettingsManager.getShouldWipeSettings()`)
    /// If (1) is `false` and (2) is `true`, we know that the app has been freshly installed/reinstalled and is
    /// compatible, thus triggering a settings wipe.
    private func doWipeSettingsIfNeeded() {
        defer {
            settingsManager.setShouldWipeSettings()
        }
        guard !self.appPreferences.hasDoneFirstTimeLaunch else { return }

        if !settingsManager.getShouldWipeSettings() {
            // the app has never been launched
            try? settingsManager.writeSettings(LatestTunnelSettings())
        } else {
            // the app has been launched after a reinstall
            if let deviceState = try? settingsManager.readDeviceState(),
                let accountData = deviceState.accountData,
                let deviceData = deviceState.deviceData
            {
                // this part is asynchronous, but we don't await the result
                _ = self.devicesProxy.deleteDevice(
                    accountNumber: accountData.number,
                    identifier: deviceData.identifier,
                    retryStrategy: .noRetry
                ) { _ in
                    // Do nothing.
                }
            }

            settingsManager.resetStore(policy: .all)
            try? settingsManager.writeSettings(LatestTunnelSettings())

            // Default access methods need to be repopulated again after settings wipe.
            self.accessMethodRepository.addDefaultsMethods()
            // At app startup, the relay cache tracker will get populated with a list of overriden IPs.
            // The overriden IPs will get wiped, therefore, the cache needs to be pruned as well.
            try? self.relayCacheTracker.refreshCachedRelays()
        }
    }

    private func doGetDefaultLocation() async {
        guard !self.appPreferences.hasDoneFirstTimeLaunch else {
            return
        }
        self.appPreferences.hasDoneFirstTimeLaunch = true
        _ = try? await relayCacheTracker.updateRelays()
        guard let cachedRelays = try? relayCacheTracker.getCachedRelays() else { return }
        let locationService = DefaultLocationService(
            urlSession: URLSession.shared, relayCache: cachedRelays)
        let locationIdentifier = try? await locationService.fetchCurrentLocationIdentifier()
        let userSelectedRelays: UserSelectedRelays =
            if let country = locationIdentifier?.country {
                UserSelectedRelays(locations: [.country(country)])
            } else {
                .default
            }

        let constraint = RelayConstraint.only(userSelectedRelays)

        if !self.appPreferences.hasDoneFirstTimeLogin {
            self.tunnelManager.updateSettings([
                .relayConstraints(RelayConstraints(entryLocations: constraint, exitLocations: constraint))
            ])
        }
    }

    private func doResolveDeprecatedSettings() async {
        await withCheckedContinuation { continuation in
            DispatchQueue.main.async { [self] in
                let resetVisibility: @Sendable () -> Void = {
                    self.appPreferences.migratedSettingsState = MigratedSettingsState(
                        preMigrationSettings: nil,
                        lastInstalledVersion: Bundle.main.productVersion,
                        lastMigratedVersion: MigratedVersion.current.rawValue,
                        hasCompletedMigrationWizard: true,
                        shouldShowMigratedSettingsMenuItem: false)
                    self.migratedSettingsListener.onMigratedSettingsHandler?(.noChanges)
                }
                MainActor.assumeIsolated {
                    let settingsResolver = DeprecatedSettingsResolver(
                        cacheDirectory: containerURL,
                        settingsManager: settingsManager,
                        relaySelector: relaySelector,
                        currentVersion: MigratedVersion(
                            rawValue: appPreferences.migratedSettingsState.lastMigratedVersion)
                            ?? MigratedVersion.current)

                    let isAppUpdated =
                        Bundle.main.productVersion != self.appPreferences.migratedSettingsState.lastInstalledVersion

                    // Reset the migrated settings menu visibility after an app update.
                    // The menu item should only remain visible within the same app version.
                    appPreferences.migratedSettingsState.shouldShowMigratedSettingsMenuItem =
                        isAppUpdated
                        ? false : self.appPreferences.migratedSettingsState.shouldShowMigratedSettingsMenuItem
                    appPreferences.migratedSettingsState.hasCompletedMigrationWizard =
                        isAppUpdated ? true : self.appPreferences.migratedSettingsState.hasCompletedMigrationWizard

                    settingsResolver.resolve(store: self.settingsManager.store) { [self] result in
                        switch result {
                        case let .migrated(old, new, changes):
                            // Tell the tunnel to re-read tunnel configuration after migration.
                            logger.debug("Successful resolving deprecated settings from UI Process")
                            appPreferences.migratedSettingsState = MigratedSettingsState(
                                preMigrationSettings: old,
                                lastInstalledVersion: Bundle.main.productVersion,
                                lastMigratedVersion: MigratedVersion.current.rawValue,
                                hasCompletedMigrationWizard: changes.isEmpty,
                                shouldShowMigratedSettingsMenuItem: !changes.isEmpty)
                            migratedSettingsListener.onMigratedSettingsHandler?(
                                changes.isEmpty ? .noChanges : .migrated)
                            tunnelManager.updateSettings([.all(new)])
                            continuation.resume(returning: ())

                        case .nothing:
                            logger.debug(
                                "Attempted resolving deprecated settings from UI Process, but It's already up to date, so nothing to do"
                            )
                            migratedSettingsListener.onMigratedSettingsHandler?(.noChanges)
                            continuation.resume(returning: ())
                        case let .failure(error):
                            logger.error("Failed resolving deprecated settings from UI Process: \(error)")
                            resetVisibility()
                            continuation.resume(returning: ())
                        }
                    }
                }
            }
        }
    }

    // MARK: - UNUserNotificationCenterDelegate

    nonisolated func userNotificationCenter(
        _ center: UNUserNotificationCenter,
        didReceive response: UNNotificationResponse,
        withCompletionHandler completionHandler: @escaping () -> Void
    ) {
        nonisolated(unsafe) let nonisolatedResponse = response
        nonisolated(unsafe) let nonisolatedCompletionHandler = completionHandler

        let blockOperation = AsyncBlockOperation(dispatchQueue: .main) {
            NotificationManager.shared.handleSystemNotificationResponse(nonisolatedResponse)

            nonisolatedCompletionHandler()
        }

        operationQueue.addOperation(blockOperation)
    }

    nonisolated func userNotificationCenter(
        _ center: UNUserNotificationCenter,
        willPresent notification: UNNotification,
        withCompletionHandler completionHandler: @escaping (UNNotificationPresentationOptions) -> Void
    ) {
        completionHandler([.list, .banner, .sound])
    }
}
