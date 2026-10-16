import XCTest
@testable import AccessibilityCore

final class ScreenReaderCursorTests: XCTestCase {
    func testEmptyCursorNeverSelectsAnItem() {
        var cursor = ScreenReaderCursor(count: 0)
        XCTAssertNil(cursor.first())
        XCTAssertNil(cursor.next())
        XCTAssertNil(cursor.previous())
        XCTAssertFalse(cursor.hasItem)
    }

    func testSingleItemCursorDoesNotWrap() {
        var cursor = ScreenReaderCursor(count: 1)
        XCTAssertEqual(cursor.first(), 0)
        XCTAssertNil(cursor.next())
        XCTAssertNil(cursor.previous())
        XCTAssertEqual(cursor.first(), 0)
    }

    func testMultipleItemsStopAtBothBoundariesAndFirstResets() {
        var cursor = ScreenReaderCursor(count: 2)
        XCTAssertEqual(cursor.first(), 0)
        XCTAssertEqual(cursor.next(), 1)
        XCTAssertNil(cursor.next())
        XCTAssertEqual(cursor.current, 1)
        XCTAssertEqual(cursor.previous(), 0)
        XCTAssertNil(cursor.previous())
        XCTAssertEqual(cursor.first(), 0)
        XCTAssertEqual(cursor.current, 0)
    }
}
