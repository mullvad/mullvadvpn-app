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
    let passthroughSubject: CurrentValueSubject<[PersistentAccessMethod], Never> = CurrentValueSubject([
        PersistentAccessMethod(
            id: UUID(uuidString: "C9DB7457-2A55-42C3-A926-C07F82131994")!,
            name: "",
            isEnabled: true,
            proxyConfiguration: .direct
        ),
        PersistentAccessMethod(
            id: UUID(uuidString: "8586E75A-CA7B-4432-B70D-EE65F3F95084")!,
            name: "",
            isEnabled: true,
            proxyConfiguration: .bridges
        ),
    ])

    public var publisher: AnyPublisher<[PersistentAccessMethod], Never> {
        passthroughSubject.eraseToAnyPublisher()
    }

    public var accessMethods: [PersistentAccessMethod] {
        passthroughSubject.value
    }

    public static let shared = AccessMethodRepository()

    private init() {
        add(passthroughSubject.value)
    }

    public func add(_ method: PersistentAccessMethod) {
        add([method])
    }

    public func add(_ methods: [PersistentAccessMethod]) {
        var storedMethods = fetchAll()

        methods.forEach { method in
            guard !storedMethods.contains(where: { $0.id == method.id }) else { return }
            storedMethods.append(method)
        }

        do {
            try writeApiAccessMethods(storedMethods)
        } catch {
            print("Could not add access method(s): \(methods) \nError: \(error)")
        }
    }

    public func update(_ method: PersistentAccessMethod) {
        var methods = fetchAll()

        guard let index = methods.firstIndex(where: { $0.id == method.id }) else { return }
        methods[index] = method

        do {
            try writeApiAccessMethods(methods)
        } catch {
            print("Could not update access method: \(method) \nError: \(error)")
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
