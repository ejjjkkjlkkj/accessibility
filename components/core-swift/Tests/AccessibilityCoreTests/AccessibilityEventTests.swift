import XCTest
@testable import AccessibilityCore

final class AccessibilityEventTests: XCTestCase {
    func testValidEvent() {
        XCTAssertTrue(AccessibilityEvent(role: .button, name: "Save").isValid)
    }

    func testInvalidABIAndOversizedName() {
        XCTAssertFalse(AccessibilityEvent(abiVersion: 2, role: .button, name: "Save").isValid)
        XCTAssertFalse(AccessibilityEvent(role: .button, name: String(repeating: "x", count: 257)).isValid)
    }
}
