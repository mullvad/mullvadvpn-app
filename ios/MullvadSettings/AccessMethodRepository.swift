//
//  AccessMethodRepository.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 12/12/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Combine
import Foundation
import MullvadLogging
import MullvadTypes

public class AccessMethodRepository: AccessMethodRepositoryProtocol {
    private let logger = Logger(label: "AccessMethodRepository")

    private let direct = PersistentAccessMethod(
        id: UUID(uuidString: "C9DB7457-2A55-42C3-A926-C07F82131994")!,
        name: "Direct",
        isEnabled: true,
        proxyConfiguration: .direct
    )

    private let bridge = PersistentAccessMethod(
        id: UUID(uuidString: "8586E75A-CA7B-4432-B70D-EE65F3F95084")!,
        name: "Mullvad bridges",
        isEnabled: true,
        proxyConfiguration: .bridges
    )

    private let encryptedDNS = PersistentAccessMethod(
        id: UUID(uuidString: "831CB1F8-1829-42DD-B9DC-82902F298EC0")!,
        name: "Encrypted DNS proxy",
        isEnabled: true,
        proxyConfiguration: .encryptedDNS
    )

    private let accessMethodsSubject: CurrentValueSubject<[PersistentAccessMethod], Never>
    public var accessMethodsPublisher: AnyPublisher<[PersistentAccessMethod], Never> {
        accessMethodsSubject.eraseToAnyPublisher()
    }

    private let lastReachableAccessMethodSubject: CurrentValueSubject<PersistentAccessMethod, Never>
    public var lastReachableAccessMethodPublisher: AnyPublisher<PersistentAccessMethod, Never> {
        lastReachableAccessMethodSubject.eraseToAnyPublisher()
    }

    public var directAccess: PersistentAccessMethod {
        direct
    }

    public init() {
        accessMethodsSubject = CurrentValueSubject([])
        lastReachableAccessMethodSubject = CurrentValueSubject(direct)

        addDefaultsMethods()

        accessMethodsSubject.send(fetchAll())
        lastReachableAccessMethodSubject.send(fetchLastReachable())
    }

    public func save(_ method: PersistentAccessMethod) {
        var methodStore = readApiAccessMethodStore()

        var method = method
        method.name = method.name.trimmingCharacters(in: .whitespaces)

        if let index = methodStore.accessMethods.firstIndex(where: { $0.id == method.id }) {
            methodStore.accessMethods[index] = method
        } else {
            methodStore.accessMethods.append(method)
        }

        do {
            try writeApiAccessMethodStore(methodStore)
            accessMethodsSubject.send(methodStore.accessMethods)
        } catch {
            logger.error("Could not save access method: \(method) \nError: \(error)")
        }
    }

    public func saveLastReachable(_ method: PersistentAccessMethod) {
        var methodStore = readApiAccessMethodStore()
        methodStore.lastReachableAccessMethod = method

        do {
            try writeApiAccessMethodStore(methodStore)
            lastReachableAccessMethodSubject.send(method)
        } catch {
            logger.error("Could not save last reachable access method: \(method) \nError: \(error)")
        }
    }

    public func delete(id: UUID) {
        var methodStore = readApiAccessMethodStore()
        guard let index = methodStore.accessMethods.firstIndex(where: { $0.id == id }) else { return }

        // Prevent removing methods that have static UUIDs and are always present.
        let method = methodStore.accessMethods[index]
        if !method.kind.isPermanent {
            methodStore.accessMethods.remove(at: index)
        }

        do {
            try writeApiAccessMethodStore(methodStore)
            accessMethodsSubject.send(methodStore.accessMethods)
        } catch {
            logger.error("Could not delete access method with id: \(id) \nError: \(error)")
        }
    }

    public func fetch(by id: UUID) -> PersistentAccessMethod? {
        fetchAll().first { $0.id == id }
    }

    public func fetchAll() -> [PersistentAccessMethod] {
        #if DEBUG
        readApiAccessMethodStore().accessMethods
        #else
        readApiAccessMethodStore().accessMethods.filter { $0.id != encryptedDNS.id }
        #endif
    }

    public func fetchLastReachable() -> PersistentAccessMethod {
        readApiAccessMethodStore().lastReachableAccessMethod
    }

    public func addDefaultsMethods() {
        #if DEBUG
        add([
            direct,
            bridge,
            encryptedDNS,
        ])
        #else
        add([
            direct,
            bridge,
        ])
        #endif
    }

    private func add(_ methods: [PersistentAccessMethod]) {
        var methodStore = readApiAccessMethodStore()

        methods.forEach { method in
            if !methodStore.accessMethods.contains(where: { $0.id == method.id }) {
                methodStore.accessMethods.append(method)
            }
        }

        do {
            try writeApiAccessMethodStore(methodStore)
            accessMethodsSubject.send(methods)
        } catch {
            logger.error("Could not update access methods: \(methods) \nError: \(error)")
        }
    }

    private func readApiAccessMethodStore() -> PersistentAccessMethodStore {
        let parser = makeParser()

        do {
            let data = try SettingsManager.store.read(key: .apiAccessMethods)
            return try parser.parseUnversionedPayload(as: PersistentAccessMethodStore.self, from: data)
        } catch {
            logger.error("Could not load access method store: \(error)")
            return PersistentAccessMethodStore(lastReachableAccessMethod: direct, accessMethods: [])
        }
    }

    private func writeApiAccessMethodStore(_ store: PersistentAccessMethodStore) throws {
        let parser = makeParser()
        let data = try parser.produceUnversionedPayload(store)

        try SettingsManager.store.write(data, for: .apiAccessMethods)
    }

    private func makeParser() -> SettingsParser {
        SettingsParser(decoder: JSONDecoder(), encoder: JSONEncoder())
    }
}
