//
//  AccountExpiryRefresh.swift
//  MullvadVPN
//
//  Created by pronebird on 28/05/2019.
//  Copyright © 2019 Amagicom AB. All rights reserved.
//

import Foundation
import os.log
import ProcedureKit

private let kRefreshIntervalSeconds: TimeInterval = 60

/// A class that manages the periodic account expiry updates.
/// All public methods are thread safe.
class AccountExpiryRefresh {

    /// A singleton instance of the AccountExpiryRefresh
    static let shared = AccountExpiryRefresh()

    private let procedureQueue: ProcedureQueue = {
        let queue = ProcedureQueue()
        queue.qualityOfService = .utility
        return queue
    }()

    /// Recursive lock used to manipulate observers
    private let lock = NSRecursiveLock()
    private var observers = [WeakBox<Observer>]()

    private init() {}

    /// Adds the observer for periodic account expiry updates.
    func startMonitoringUpdates(with block: @escaping (Date) -> Void) -> Observer {
        let observer = Observer(with: block)

        addObserver(observer)

        return observer
    }

    /// Register observer and start updating the account expiry if hasn't started yet
    private func addObserver(_ observer: Observer) {
        lock.withCriticalScope {
            let wasEmpty = observers.isEmpty

            observers.append(WeakBox(observer))

            if wasEmpty {
                procedureQueue.addOperation(makePeriodicUpdateProcedure())
            }
        }

    }

    /// Remove all boxed values whos underlying weak value has been released
    private func compactObservers() {
        lock.withCriticalScope {
            observers.removeAll { $0.unboxed == nil }
        }
    }

    /// Broadcast the new expiry to the observers
    private func notifyObservers(with newExpiry: Date) {
        let strongObservers = lock.withCriticalScope { observers.compactMap { $0.unboxed } }

        DispatchQueue.main.async {
            strongObservers.forEach { $0.notify(with: newExpiry) }
        }
    }

    /// Returns true if the repeat procedure should keep running
    private func shouldKeepRefreshing() -> Bool {
        return lock.withCriticalScope {
            compactObservers()

            return !observers.isEmpty
        }
    }

    /// Create a procedure that will repeat itself with a constant interval until there are no
    /// observers left.
    private func makePeriodicUpdateProcedure() -> RepeatProcedure<Operation> {
        let repeatProcedure = RepeatProcedure(wait: .constant(kRefreshIntervalSeconds)) { [weak self] () -> Operation? in
            // Stop repeating the procedure if no-one is listening
            guard let self = self, self.shouldKeepRefreshing() else { return nil }

            // Create the procedure to feed the account token saved in preferences into the
            // request procedure
            let getAccountTokenProcedure = ResultProcedure(block: { Account.token })

            // Create the API request procedure
            let requestProcedure = MullvadAPI.getAccountExpiry()
                .injectResult(from: getAccountTokenProcedure)

            // Create the procedure to save the received account expiry and notify the observers
            let saveAccountExpiryProcedure = TransformProcedure { [weak self] (response) throws -> Void in
                let userDefaultsInteractor = UserDefaultsInteractor.sharedApplicationGroupInteractor

                let newAccountExpiry = try response.result.get()
                let oldAccountExpiry = userDefaultsInteractor.accountExpiry

                if oldAccountExpiry != newAccountExpiry {
                    userDefaultsInteractor.accountExpiry = newAccountExpiry

                    self?.notifyObservers(with: newAccountExpiry)
                }
                }.injectResult(from: requestProcedure)

            // Return the group
            return GroupProcedure(operations: [getAccountTokenProcedure, requestProcedure, saveAccountExpiryProcedure])
        }

        // Make sure that only one such operation runs at a time
        repeatProcedure.addCondition(MutuallyExclusive<AccountExpiryRefresh>())

        return repeatProcedure
    }

    /// The account expiry observer.
    class Observer {
        typealias Block = (Date) -> Void
        private let block: Block

        fileprivate init(with block: @escaping Block) {
            self.block = block
        }

        fileprivate func notify(with expiryDate: Date) {
            block(expiryDate)
        }
    }

}
