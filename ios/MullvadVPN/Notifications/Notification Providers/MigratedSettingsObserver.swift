// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import MullvadSettings
import MullvadTypes

//MARK: - MigratedSettingsPropagation
enum MigratedSettingsResult: Equatable, Sendable {
    case noChanges
    case migrated
}
protocol MigratedSettingsPropagation: Sendable {
    typealias MigratedSettingsHandler = @Sendable (MigratedSettingsResult) -> Void
    var onMigratedSettingsHandler: MigratedSettingsHandler? { get set }
}

protocol MigratedSettingsObserver: AnyObject {
    func didUpdateMigratedSettingsState(_ result: MigratedSettingsResult)
}

class MigratedSettingsObserverBlock: MigratedSettingsObserver {
    typealias DidUpdateMigratedSettingsStateHandler = (MigratedSettingsResult) -> Void
    var onNewMigratedSettingsState: DidUpdateMigratedSettingsStateHandler

    init(didMigratedSettingsState: @escaping DidUpdateMigratedSettingsStateHandler) {
        self.onNewMigratedSettingsState = didMigratedSettingsState
    }

    func didUpdateMigratedSettingsState(_ result: MigratedSettingsResult) {
        self.onNewMigratedSettingsState(result)
    }
}

final class MigratedSettingsListener: MigratedSettingsPropagation, @unchecked Sendable {
    var onMigratedSettingsHandler: MigratedSettingsHandler?

    init(onMigratedSettingsHandler: MigratedSettingsHandler? = nil) {
        self.onMigratedSettingsHandler = onMigratedSettingsHandler
    }
}

final class MigratedSettingsUpdater: Sendable {
    /// Observers.
    private let observerList = ObserverList<MigratedSettingsObserver>()
    nonisolated(unsafe) private var listener: MigratedSettingsPropagation

    init(listener: MigratedSettingsPropagation) {
        self.listener = listener
        self.listener.onMigratedSettingsHandler = { [weak self] settings in
            guard let self else { return }
            self.observerList.notify {
                $0.didUpdateMigratedSettingsState(settings)
            }
        }
    }

    // MARK: - Multihop observations

    func addObserver(_ observer: MigratedSettingsObserver) {
        observerList.append(observer)
    }

    func removeObserver(_ observer: MigratedSettingsObserver) {
        observerList.remove(observer)
    }
}
