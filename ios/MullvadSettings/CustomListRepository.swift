//
//  CustomListRepository.swift
//  MullvadVPN
//
//  Created by Mojgan on 2024-01-25.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Combine
import Foundation
import MullvadLogging
import MullvadTypes

public enum CustomRelayListError: LocalizedError, Hashable {
    case duplicateName
    case nameTooLong

    public var errorDescription: String? {
        switch self {
        case .duplicateName:
            NSLocalizedString("Name is already taken.", comment: "")
        case .nameTooLong:
            String(
                format: NSLocalizedString("Name should be no longer than %i characters.", comment: ""),
                NameInputFormatter.maxLength
            )
        }
    }
}

public struct CustomListRepository: CustomListRepositoryProtocol {
    private let logger = Logger(label: "CustomListRepository")

    private let settingsParser: SettingsParser = {
        SettingsParser(decoder: JSONDecoder(), encoder: JSONEncoder())
    }()

    let store: SettingsStore

    public init(settingsStore: SettingsStore) {
        self.store = settingsStore
    }

    public func save(list: CustomList) throws {
        guard list.name.count <= NameInputFormatter.maxLength else {
            throw CustomRelayListError.nameTooLong
        }

        var lists = fetchAll()

        var list = list
        list.name = list.name.trimmingCharacters(in: .whitespaces)

        if let listWithSameName = lists.first(where: { $0.name.compare(list.name) == .orderedSame }),
            listWithSameName.id != list.id
        {
            throw CustomRelayListError.duplicateName
        } else if let index = lists.firstIndex(where: { $0.id == list.id }) {
            lists[index] = list
            try write(lists, to: store)
        } else {
            lists.append(list)
            try write(lists, to: store)
        }
    }

    public func delete(id: UUID) {
        do {
            var lists = fetchAll()
            if let index = lists.firstIndex(where: { $0.id == id }) {
                lists.remove(at: index)
                try write(lists, to: store)
            }
        } catch {
            logger.error(error: error)
        }
    }

    public func fetch(by id: UUID) -> CustomList? {
        try? read(from: store).first(where: { $0.id == id })
    }

    public func fetchAll() -> [CustomList] {
        (try? read(from: store)) ?? []
    }
}

extension CustomListRepository {
    private func read(from store: SettingsStore) throws -> [CustomList] {
        let data = try store.read(key: .customRelayLists)

        return try settingsParser.parseUnversionedPayload(as: [CustomList].self, from: data)
    }

    private func write(_ list: [CustomList], to store: SettingsStore) throws {
        let data = try settingsParser.produceUnversionedPayload(list)

        try store.write(data, for: .customRelayLists)
    }
}
