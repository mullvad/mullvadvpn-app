// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import Combine
import MullvadLogging
import MullvadSettings
import MullvadTypes

final class IPOverrideInteractor: @unchecked Sendable {
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

    func `import`(url: URL) async {
        let data = (try? Data(contentsOf: url)) ?? Data()
        await handleImport(of: data, context: .file(fileName: url.lastPathComponent))
    }

    func `import`(text: String) async {
        let data = text.data(using: .utf8) ?? Data()
        await handleImport(of: data, context: .text)
    }

    func deleteAllOverrides() async {
        repository.deleteAll()

        updateTunnel()
        resetToDefaultStatus()
    }

    private func handleImport(of data: Data, context: IPOverrideStatus.Context) async {
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
        tunnelManager.refreshRelayCacheAndReconnectIfNeeded { [weak self] error in
            if let error {
                self?.logger.error(error: error, message: "Could not refresh relay cache tracker.")
            }
        }
    }

    private func resetToDefaultStatus(delay: Duration = .zero) {
        statusWorkItem?.cancel()

        let statusWorkItem = DispatchWorkItem { [weak self] in
            guard let self, self.statusWorkItem?.isCancelled == false else { return }
            self.statusSubject.send(self.defaultStatus)
        }
        self.statusWorkItem = statusWorkItem

        DispatchQueue.main.asyncAfter(deadline: .now() + delay.timeInterval, execute: statusWorkItem)
    }
}
