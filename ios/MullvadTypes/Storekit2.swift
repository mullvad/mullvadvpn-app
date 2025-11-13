public struct StoreKitTransaction: Codable, Sendable {
    let transaction: String

    public init(transaction: String) {
        self.transaction = transaction
    }
}

public struct LegacyStoreKitRequest: Codable, Sendable {
    let receiptString: Data

    public init(receiptString: Data) {
        self.receiptString = receiptString
    }
}
