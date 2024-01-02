//
//  AccessMethodRepository.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 12/12/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Combine
import Foundation

public class AccessMethodRepository: AccessMethodRepositoryProtocol {
    private let direct = PersistentAccessMethod(
        id: UUID(uuidString: "C9DB7457-2A55-42C3-A926-C07F82131994")!,
        name: "",
        isEnabled: true,
        proxyConfiguration: .direct
    )

    private let bridge = PersistentAccessMethod(
        id: UUID(uuidString: "8586E75A-CA7B-4432-B70D-EE65F3F95084")!,
        name: "",
        isEnabled: true,
        proxyConfiguration: .bridges
    )

    let passthroughSubject: CurrentValueSubject<[PersistentAccessMethod], Never> = CurrentValueSubject([])

    public var publisher: AnyPublisher<[PersistentAccessMethod], Never> {
        passthroughSubject.eraseToAnyPublisher()
    }

    public var directAccess: PersistentAccessMethod {
        direct
    }

    public init() {
        add([direct, bridge])
    }

    public func save(_ method: PersistentAccessMethod) {
        var storedMethods = fetchAll()

        if let index = storedMethods.firstIndex(where: { $0.id == method.id }) {
            storedMethods[index] = method
        } else {
            storedMethods.append(method)
        }

        do {
            try writeApiAccessMethods(storedMethods)
        } catch {
            print("Could not update access methods: \(storedMethods) \nError: \(error)")
        }
    }

    public func delete(id: UUID) {
        var methods = fetchAll()
        guard let index = methods.firstIndex(where: { $0.id == id }) else { return }

        // Prevent removing methods that have static UUIDs and are always present.
        let method = methods[index]
        if !method.kind.isPermanent {
            methods.remove(at: index)
        }

        do {
            try writeApiAccessMethods(methods)
        } catch {
            print("Could not delete access method with id: \(id) \nError: \(error)")
        }
    }

    public func fetch(by id: UUID) -> PersistentAccessMethod? {
        fetchAll().first { $0.id == id }
    }

    public func fetchAll() -> [PersistentAccessMethod] {
        (try? readApiAccessMethods()) ?? []
    }

    public func reloadWithDefaultsAfterDataRemoval() {
        add([direct, bridge])
    }

    private func add(_ methods: [PersistentAccessMethod]) {
        var storedMethods = fetchAll()

        passthroughSubject.value.forEach { method in
            if !storedMethods.contains(where: { $0.id == method.id }) {
                storedMethods.append(method)
            }
        }

        do {
            try writeApiAccessMethods(storedMethods)
        } catch {
            print("Could not update access methods: \(storedMethods) \nError: \(error)")
        }
    }

    private func readApiAccessMethods() throws -> [PersistentAccessMethod] {
        let parser = makeParser()
        let data = try SettingsManager.store.read(key: .apiAccessMethods)

        return try parser.parseUnversionedPayload(as: [PersistentAccessMethod].self, from: data)
    }

    private func writeApiAccessMethods(_ accessMethods: [PersistentAccessMethod]) throws {
        let parser = makeParser()
        let data = try parser.produceUnversionedPayload(accessMethods)

        try SettingsManager.store.write(data, for: .apiAccessMethods)

        passthroughSubject.send(accessMethods)
    }

    private func makeParser() -> SettingsParser {
        SettingsParser(decoder: JSONDecoder(), encoder: JSONEncoder())
    }
}
