import XCTest

final class DemlemeUITests: XCTestCase {
    override func setUpWithError() throws {
        continueAfterFailure = false
    }

    @MainActor
    func testLaunchShowsTitle() throws {
        let app = XCUIApplication()
        app.launch()
        let title = app.staticTexts["app.title"]
        XCTAssertTrue(title.waitForExistence(timeout: 5))
        XCTAssertEqual(title.label, "Demleme")
    }
}
