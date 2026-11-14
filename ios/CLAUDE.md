# ios

## What it does
- SwiftUI app (iPhone, iPad; iOS 18+, Swift 6.0).
- XcodeGen generates the Xcode project from `project.yml`. The targets are `Demleme`, `DemlemeTests` and `DemlemeUITests`.
- The entry point and the first screen are in `Demleme/App`.

## What it does not touch
- Do not edit `.xcodeproj` by hand. Do not add it to the repository. Make changes in `project.yml`.
- It has no business rules and no calculation. The server does the calculation. The iOS app only draws the interface.
- The product owner and CI decide the bundle id and signing (`DEVELOPMENT_TEAM`). Do not set them here.

## How to test
- Linux: `python3 -I ios/scripts/validate.py` (Swift does not compile on Linux).
- Check that the generated colors are current: `python3 -I ios/scripts/gen_colors.py --check`.
- The macOS CI does the build and the tests. The product owner does the device test.

## Rules
- Do not edit generated files by hand: `Demleme/Resources/Assets.xcassets/Colors` and `Demleme/Design/Tokens.swift`. Generate them again with `python3 -I ios/scripts/gen_colors.py`.
- Format: SwiftFormat and SwiftLint (`.swiftformat`, `.swiftlint.yml`).
- Code is in English. Notes for Claude (this file, task lists) use ASD-STE100. Other documents are in Turkish. User-visible text comes from the text catalog.
- Start the commit message with the task number: `T0.x: ...`.
