public struct StorekitTransaction: Codable, Sendable {
    let transaction: String

    public init(transaction: String) {
        self.transaction = transaction
    }
}

public struct LegacyStorekitRequest: Codable, Sendable {
    let receiptString: Data

    public init(receiptString: Data) {
        self.receiptString = receiptString
    }
}
