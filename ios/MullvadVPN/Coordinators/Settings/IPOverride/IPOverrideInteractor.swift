//
//  IPOverrideInteractor.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-01-30.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Combine
import MullvadLogging
import MullvadSettings
import MullvadTypes

final class IPOverrideInteractor {
    private let logger = Logger(label: "IPOverrideInteractor")
    private let repository: IPOverrideRepositoryProtocol
    private let tunnelManager: TunnelManager
    private var statusWorkItem: DispatchWorkItem?

    private let statusSubject = CurrentValueSubject<IPOverrideStatus, Never>(.noImports)
    var statusPublisher: AnyPublisher<IPOverrideStatus, Never> {
        statusSubject.eraseToAnyPublisher()
    }

    var defaultStatus: IPOverrideStatus {
        if repository.fetchAll().isEmpty {
            return .noImports
        } else {
            return .active
        }
    }

    init(repository: IPOverrideRepositoryProtocol, tunnelManager: TunnelManager) {
        self.repository = repository
        self.tunnelManager = tunnelManager

        resetToDefaultStatus()
    }

    func `import`(url: URL) {
        let data = (try? Data(contentsOf: url)) ?? Data()
        handleImport(of: data, context: .file)
    }

    func `import`(text: String) {
        let data = text.data(using: .utf8) ?? Data()
        handleImport(of: data, context: .text)
    }

    func deleteAllOverrides() {
        repository.deleteAll()

        updateTunnel()
        resetToDefaultStatus()
    }

    private func handleImport(of data: Data, context: IPOverrideStatus.Context) {
        do {
            let overrides = try repository.parse(data: data)

            repository.add(overrides)
            statusSubject.send(.importSuccessful(context))
        } catch {
            statusSubject.send(.importFailed(context))
            logger.error("Error importing ip overrides: \(error)")
        }

        updateTunnel()

        // After an import - successful or not - the UI should be reset back to default
        // state after a certain amount of time.
        resetToDefaultStatus(delay: .seconds(10))
    }

    private func updateTunnel() {
        do {
            try tunnelManager.refreshRelayCacheTracker()
        } catch {
            logger.error(error: error, message: "Could not refresh relay cache tracker.")
        }

        switch tunnelManager.tunnelStatus.observedState {
        case .connecting, .connected, .reconnecting:
            tunnelManager.reconnectTunnel(selectNewRelay: true)
        default:
            break
        }
    }

    private func resetToDefaultStatus(delay: Duration = .zero) {
        statusWorkItem?.cancel()

        let statusWorkItem = DispatchWorkItem { [weak self] in
            let isCancelled = self?.statusWorkItem?.isCancelled ?? false
            guard let self, !isCancelled else { return }

            self.statusSubject.send(self.defaultStatus)
        }
        self.statusWorkItem = statusWorkItem

        DispatchQueue.main.asyncAfter(deadline: .now() + delay.timeInterval, execute: statusWorkItem)
    }
}
