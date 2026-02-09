from playwright.sync_api import sync_playwright

def verify_header_padding():
    with sync_playwright() as p:
        browser = p.chromium.launch(headless=True)
        # Use a large enough viewport to ensure filters are visible (md breakpoint)
        context = browser.new_context(viewport={'width': 1280, 'height': 800})
        page = context.new_page()

        try:
            # 1. Navigate to the app
            page.goto("http://localhost:5173")

            # 2. Wait for the header elements to be visible
            page.get_by_placeholder("検索...").wait_for()

            # 3. Take a screenshot of the initial state (before padding change)
            # This is technically 'after' the previous submit, so it has py-2
            page.screenshot(path="verification/header_padding_before.png")

            # Let's inspect the bounding box height of the search container vs filter container

            # Filter container (the glass pill on the left)
            # It's inside the AnimatePresence.
            # We can find it by class: "pointer-events-auto flex items-center rounded-full"
            # Or by text content of one of the selects.

            # The filter pill is the parent of the first button inside the header?
            # header > div > div > button (trigger)
            # The filter pill is the div that contains the button.

            # Use a robust locator for the filter container
            filter_pill = page.locator("header .pointer-events-auto.flex.items-center.rounded-full").first

            # Search container
            # "group relative shrink-0 overflow-hidden rounded-full"
            search_pill = page.locator("header .group.relative.shrink-0").first

            filter_box = filter_pill.bounding_box()
            search_box = search_pill.bounding_box()

            if filter_box:
                print(f"Filter Pill Height: {filter_box['height']}")
            else:
                print("Filter Pill not found")

            if search_box:
                print(f"Search Pill Height: {search_box['height']}")
            else:
                print("Search Pill not found")

        except Exception as e:
            print(f"Error: {e}")
            page.screenshot(path="verification/error_padding.png")
        finally:
            browser.close()

if __name__ == "__main__":
    verify_header_padding()
