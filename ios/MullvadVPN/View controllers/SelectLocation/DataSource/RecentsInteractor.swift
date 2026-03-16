//
//  RecentsInteractor.swift
//  MullvadVPN
//
//  Created by Mojgan on 2025-11-25.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Combine
import MullvadLogging
import MullvadSettings
import MullvadTypes

protocol RecentsInteractorProtocol {
    var isEnabledPublisher: AnyPublisher<Bool, Never> { get }
    func toggle()
    func fetch(context: MultihopContext) -> [RelayConstraint<UserSelectedRelays>]
    func updateSelectedLocations(_ constraint: RelayConstraint<UserSelectedRelays>, for context: MultihopContext)
    func cleanup(_ customList: UUID)
}

class RecentsInteractor: RecentsInteractorProtocol {
    private let repository: RecentConnectionsRepositoryProtocol
    private var selectedEntryConstraint: RelayConstraint<UserSelectedRelays>
    private var selectedExitConstraint: RelayConstraint<UserSelectedRelays>
    private let logger = Logger(label: "RecentsInteractor")
    private var recentConnections: RecentConnections?
    private var cancellables = Set<Combine.AnyCancellable>()
    private var isEnabledSubject = CurrentValueSubject<Bool, Never>(false)
    var isEnabledPublisher: AnyPublisher<Bool, Never> {
        isEnabledSubject.eraseToAnyPublisher()
    }

    init(
        selectedEntryConstraint: RelayConstraint<UserSelectedRelays>,
        selectedExitConstraint: RelayConstraint<UserSelectedRelays>,
        repository: RecentConnectionsRepositoryProtocol
    ) {
        self.repository = repository
        self.selectedEntryConstraint = selectedEntryConstraint
        self.selectedExitConstraint = selectedExitConstraint
        self.subscribeToRecentConnections()
        self.repository.load()
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
                    repository.enable(selectedEntryConstraint, selectedExitConstraint: selectedExitConstraint)
                case .failure(let error):
                    logger.error("Failed to subscribe to recent connections: \(error)")
                }
            })
            .store(in: &cancellables)
    }

    func updateSelectedLocations(
        _ selectedConstraint: RelayConstraint<UserSelectedRelays>, for context: MultihopContext
    ) {
        updateRelays(selectedConstraint, for: context)
        guard isEnabled else { return }
        persistRelaySelection()
    }

    var isEnabled: Bool {
        recentConnections?.isEnabled ?? false
    }

    func fetch(context: MultihopContext) -> [RelayConstraint<UserSelectedRelays>] {
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
            repository.enable(selectedEntryConstraint, selectedExitConstraint: selectedExitConstraint)
        }
    }

    private func updateRelays(
        _ constraint: RelayConstraint<UserSelectedRelays>,
        for context: MultihopContext
    ) {
        switch context {
        case .entry:
            selectedEntryConstraint = constraint
        case .exit:
            selectedExitConstraint = constraint
        }
    }

    private func persistRelaySelection() {
        repository.add(
            selectedEntryConstraint,
            selectedExitConstraint: selectedExitConstraint
        )
    }

    func cleanup(_ customList: UUID) {
        repository.deleteCustomList(customList)
    }
}
