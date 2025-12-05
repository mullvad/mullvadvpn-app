//
//  RecentsInteractor.swift
//  MullvadVPN
//
//  Created by Mojgan on 2025-11-25.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Combine
import MullvadLogging
import MullvadSettings
import MullvadTypes

protocol RecentsInteractorProtocol {
    var isEnabledPublisher: AnyPublisher<Bool, Never> { get }
    func toggle()
    func fetch(context: MultihopContext) -> [UserSelectedRelays]
    func updateSelectedLocations(_ selectedLocations: UserSelectedRelays, for context: MultihopContext)
}

class RecentsInteractor: RecentsInteractorProtocol {
    private let repository: RecentConnectionsRepositoryProtocol
    private var selectedEntryRelays: UserSelectedRelays?
    private var selectedExitRelays: UserSelectedRelays
    private let logger = Logger(label: "RecentsInteractor")
    private var recentConnections: RecentConnections?
    private var cancellables = Set<Combine.AnyCancellable>()
    private var isEnabledSubject = CurrentValueSubject<Bool, Never>(false)
    var isEnabledPublisher: AnyPublisher<Bool, Never> {
        isEnabledSubject.eraseToAnyPublisher()
    }

    init(
        selectedEntryRelays: UserSelectedRelays?,
        selectedExitRelays: UserSelectedRelays,
        repository: RecentConnectionsRepositoryProtocol
    ) {
        self.repository = repository
        self.selectedEntryRelays = selectedEntryRelays
        self.selectedExitRelays = selectedExitRelays
        self.subscribeToRecentConnections()
        self.repository.initiate()
    }

    private func subscribeToRecentConnections() {
        repository
            .recentConnectionsPublisher
            .sink(receiveValue: { [weak self] result in
                guard let self else { return }
                switch result {
                case .success(let value):
                    recentConnections = value
                    isEnabledSubject.send(value.isEnabled)
                case .failure(let error) where (error as? KeychainError) == .itemNotFound:
                    // Key not found: this occurs only on first use.
                    // Initialize Recents using the user's most recent entry/exit selections by default.
                    repository.enable(selectedEntryRelays, selectedExitRelays: selectedExitRelays)
                case .failure(let error):
                    logger.error("Failed to subscribe to recent connections: \(error)")
                }
            })
            .store(in: &cancellables)
    }

    func updateSelectedLocations(_ selectedLocations: UserSelectedRelays, for context: MultihopContext) {
        updateRelays(selectedLocations, for: context)
        guard isEnabled else { return }
        persistRelaySelection()
    }

    var isEnabled: Bool {
        recentConnections?.isEnabled ?? false
    }

    func fetch(context: MultihopContext) -> [UserSelectedRelays] {
        switch context {
        case .entry:
            recentConnections?.entryLocations ?? []
        case .exit:
            recentConnections?.exitLocations ?? []
        }
    }

    func toggle() {
        if isEnabled {
            repository.disable()
        } else {
            repository.enable(selectedEntryRelays, selectedExitRelays: selectedExitRelays)
        }
    }

    private func updateRelays(
        _ relays: UserSelectedRelays,
        for context: MultihopContext
    ) {
        switch context {
        case .entry:
            selectedEntryRelays = relays
        case .exit:
            selectedExitRelays = relays
        }
    }

    private func persistRelaySelection() {
        repository.add(
            selectedEntryRelays,
            selectedExitRelays: selectedExitRelays
        )
    }
}
