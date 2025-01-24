//
//  SendableTests.swift
//  MullvadVPNTests
//
//  Created by Marco Nikic on 2025-01-24.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import Testing

// Cannot be sendable unless marked `final`
class NonSendable {
    var name = "Do not send me"

    func doNonSendableThings() {
        print("really \(name)")
    }

    deinit {
        print("bye \(name)")
    }
}

struct SendableButNonCopyableValue: Sendable, ~Copyable {
    let name = "Please send me senpai :3"
}

nonisolated(unsafe) var plainSendableTypeCounter = 0

class PlainSendableType {
    var name = "I have been sent here"

    init() {
        plainSendableTypeCounter += 1

        print("I am instance #\(plainSendableTypeCounter) of PlainSendableType")
    }

    deinit {
        print("Instance #\(plainSendableTypeCounter) of PlainSendableType has been terminated")
        plainSendableTypeCounter -= 1
    }
}

struct InvalidUse {
    private let ns: NonSendable

    init(nonSendable: NonSendable) {
        self.ns = nonSendable
    }

    func sendValue() async -> sending NonSendable {
        ns.doNonSendableThings()
        // return ns // Sending 'self.ns' risks causing data races
        return NonSendable()
    }
}

struct ValidUse {
    func sendValue() async -> sending NonSendable {
        NonSendable()
    }
}

struct SendableTests {
    @Test func firstExample() async {
        let nonSendable = NonSendable()
        let invalid = InvalidUse(nonSendable: nonSendable)

        let valid = ValidUse()

        let invalidUse = await invalid.sendValue()
        let validUse = await valid.sendValue()
        print(invalidUse)
        print(validUse)
    }
}

struct ReceivingSendable {
    func receiveSending(_ nonSendable: sending NonSendable) async {
        print(nonSendable.name)
    }

    /// `borrowing + sending` is technically valid,
    /// but the Language steering group decided to ban it for now, preferring internal compiler exclusivity
    /// to preserve ABI compatibility
    /// https://forums.swift.org/t/borrowing-sending-not-allowed/74711/2
    func borrowSendingSendable(_ sendableValue: borrowing /* sending */ SendableButNonCopyableValue) async {
        print(sendableValue.name)
    }

    func consumeSendingSendable(_ sendableValue: consuming sending SendableButNonCopyableValue) async {
        print(sendableValue.name)
    }

    @Test
    mutating func secondExample() async {
        /// `nonSendable` is in a disconnected isolation region from `ReceivingSendable`, the current test instance
        let nonSendable = NonSendable()

        /// `nonSendable` is being sent into `ReceivingSendable`'s isolation region, it cannot be used anymore in the disconnected region
        /// because that is a potential race condition
        await receiveSending(nonSendable) // ❌ Sending 'nonSendable' risks causing data races
        /// Uncommenting the next line will trigger the error in the comment above
//            nonSendable.name = "Invalid operation"

        /// Is it okay to borrow `~Copyable & Sendable` values between isolated regions, no risk of race conditions occur
        let sendableValue = SendableButNonCopyableValue() // ❌'sendableValue' used after consume
        await borrowSendingSendable(sendableValue)
        print(sendableValue.name)

        /// However, even if a value is `Sendable`, if it is `~Copyable`
        /// It cannot be used across different isolated regions once it is consumed
        await consumeSendingSendable(sendableValue)
        /// Uncommenting the next line triggers the error at L82
//            print(sendableValue.name)
    }
}

class HasDelegate {
    var borrowingDelegateCallback: ((borrowing SendableButNonCopyableValue) -> Void)?

    var impossibleSendingDelegateCallback: ((consuming sending SendableButNonCopyableValue) -> Void)?

    typealias CompletionHandler = (sending Result<NonSendable, Never>) -> Void

    var completionHandler: CompletionHandler?

    func callBorrowingDelegate() {
        let argument = SendableButNonCopyableValue()
        borrowingDelegateCallback?(argument)
    }

    func callSendingDelegate() {
        let argument = SendableButNonCopyableValue()
        impossibleSendingDelegateCallback?(argument)
    }

    func callCompletionHandler() {
        let result: Result<NonSendable, Never> = Result.success(NonSendable())
        completionHandler?(result) // ❌ Sending 'result' risks causing data races

//        _ = result.map { print($0) }
    }
}

class HasDifferentDelegate {
    typealias OtherCompletionHandler = () -> sending PlainSendableType

    var otherHandler: OtherCompletionHandler?

    let plainSendableType: PlainSendableType

    init(plainSendableType: PlainSendableType) {
        self.plainSendableType = plainSendableType
    }

    func callOtherHandler() {
        let value = otherHandler!()
        print(value.name)
    }
}

class SendingClosures {
    func extractNameFrom(_ argument: borrowing SendableButNonCopyableValue) -> String {
        argument.name
    }

    func stealNameFrom(_ argument: consuming sending SendableButNonCopyableValue) -> String {
        argument.name
    }

    @Test func sendCannotBorrowAndConsume() {
        let hasDelegate = HasDelegate()

        hasDelegate.borrowingDelegateCallback = { argument in // ❌ 'argument' is borrowed and cannot be consumed
            let borrowedName = self.extractNameFrom(argument)
            print(borrowedName)

//            let stolenName = self.stealNameFrom(argument)
//            print(stolenName)
        }

        hasDelegate.callBorrowingDelegate()
    }

    @Test func nonsenseSendingCallback() {
        let hasDelegate = HasDelegate()

        hasDelegate.impossibleSendingDelegateCallback = { _ in // ❌ 'argument' is borrowed and cannot be consumed
//            let stolenName = self.stealNameFrom(argument)
//            print(stolenName)
        }

        hasDelegate.callSendingDelegate()
    }

    @Test func sendCustomCallback() {
        let hasDelegate = HasDelegate()

        hasDelegate.completionHandler = { maybeResult in
            print(maybeResult.get().name)
        }

        hasDelegate.callCompletionHandler()
    }

    @Test func sendCustomCallbackDifferently() {
        let plainType = PlainSendableType()
        let hasDelegate = HasDifferentDelegate(plainSendableType: plainType)

        hasDelegate.otherHandler = {
//            return plainType // ❌ Sending 'plainType' risks causing data races
            PlainSendableType()
        }

        plainType.name = "hello"
        hasDelegate.callOtherHandler()
    }
}
