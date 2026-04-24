//
//  LogViewModel.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2026-04-23.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import MullvadLogging

@MainActor
final class LogViewInteractor {
    var didAddEntry: ((InAppLogEntry) -> Void)?
    private let observer: InAppLogBlockObserver

    init(observer: InAppLogBlockObserver) {
        self.observer = observer

        self.observer.didAddLogEntryHandler = { entry in
            Task { @MainActor [weak self] in
                self?.didAddEntry?(entry)
            }
        }
    }
}
