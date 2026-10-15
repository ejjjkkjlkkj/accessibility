import XCTest
@testable import AccessibilityCore

final class AccessibilityEventTests: XCTestCase {
    func testValidEvent() { XCTAssertTrue(AccessibilityEvent(role: .button, name: "Save").isValid) }
    func testInvalidABIAndOversizedName() {
        XCTAssertFalse(AccessibilityEvent(abiVersion: 2, role: .button, name: "Save").isValid)
        XCTAssertFalse(AccessibilityEvent(role: .button, name: String(repeating: "x", count: 4097)).isValid)
    }
    func testCanonicalRoleCodes() { XCTAssertEqual(AccessibilityRole.button.rawValue, 5); XCTAssertEqual(AccessibilityRole.menuItem.rawValue, 17) }
    func testSemanticStateFlagValidation() {
        let focused = AccessibilityState.focused | AccessibilityState.focusable | AccessibilityState.enabled
        XCTAssertTrue(AccessibilityState.isValid(focused))
        XCTAssertTrue(AccessibilityEvent(role: .button, name: "Continue", stateFlags: focused).isValid)
        XCTAssertFalse(AccessibilityState.isValid(AccessibilityState.focused))
        XCTAssertFalse(AccessibilityState.isValid(AccessibilityState.enabled | AccessibilityState.disabled))
        XCTAssertFalse(AccessibilityState.isValid(UInt64(1) << 63))
        XCTAssertFalse(AccessibilityEvent(role: .button, name: "bad\0name").isValid)
    }
    func testScreenReaderCursorBoundaries() {
        var cursor = ScreenReaderCursor(count: 2)
        XCTAssertEqual(cursor.first(), 0); XCTAssertEqual(cursor.next(), 1)
        XCTAssertNil(cursor.next()); XCTAssertEqual(cursor.previous(), 0); XCTAssertNil(cursor.previous())
        var empty = ScreenReaderCursor(count: 0); XCTAssertNil(empty.first())
    }
}
