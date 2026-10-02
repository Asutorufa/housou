# Telegram Instant View

Anime links such as `https://housou.asutorufa.com/?day=1&anime=...` now contain a
visible, server-rendered Japanese article inside `#root`. It uses the same
metadata lookup and cache as link previews. React replaces this fallback when
the app mounts, so the normal schedule and detail dialog still work.

The article includes the cover, a full Japanese TMDb synopsis when available,
basic information, studios, cast, staff, episodes, and links to the metadata
source and the interactive website. English AniList/Jikan synopses use Japanese
fallback text. No viewer-specific account or comment data is rendered.

The template in [telegram-instant-view.template](telegram-instant-view.template)
selects the heading, cover, and article body separately to avoid duplicated
titles or images. It excludes the homepage, empty anime parameters, and metadata
lookup failures. Its selectors can be checked locally; the Telegram parser and
client still need validation in the Instant View editor.

## Later setup

1. Deploy the Worker and frontend together.
2. Sign in to [Telegram's Instant View editor](https://instantview.telegram.org/my/)
   and open a deployed anime URL.
3. Paste the template rules. Test several titles, including missing Japanese
   synopses, missing covers, and episode lists. Confirm that the homepage and
   failed lookups do not generate Instant View pages.
4. Use the editor's **View in Telegram** link to obtain the template's `rhash`.
   Share `https://t.me/iv?url=<percent-encoded-anime-url>&rhash=<template-rhash>`
   to test with that template before public approval.
5. If ordinary website links should show the Instant View button automatically,
   submit the template to Telegram for approval.

No `rhash` is fabricated or configured in this repository, and this change does
not register or submit a template. See Telegram's
[manual](https://instantview.telegram.org/docs),
[checklist](https://instantview.telegram.org/checklist), and
[publishing instructions](https://instantview.telegram.org/#publishing-templates).
