# Kompas discovery and ratings review

Research date: 2026-10-03. This is a product decision record, not a claim that the planned features are already implemented or that Kompas is better than existing stores. Source pages were checked against official documentation. No user tracking or review submission is enabled by this document.

## Product goal

A user should find a suitable app or game, understand why it is relevant and how it will run, and install or open its real storefront without learning Linux packaging. A visually attractive package list is insufficient. Bazaar already offers modern Flatpak discovery and Discover already aggregates package sources. Kompas's opportunity is relevant discovery across applications and commercial games, combined with honest system-specific availability and launcher guidance.

## What to borrow

| Store | Observed pattern | Decision for Kompas |
| --- | --- | --- |
| Apple App Store / Mac App Store | Distinct apps/games experiences, editorial collections, categories, screenshots, text relevance and behavioral signals | Use small purposeful collections such as photo editing, working with PDFs, controller games and strategy games; explain each collection's purpose. Keep manual selections distinct from measured popularity. |
| Steam | Genre/tag browsing, similar-title discovery, review recommendation percentages and review counts | Share game genres across sources. Later add tag overlap recommendations and controller/price filters when reliable metadata exists. Keep Steam sentiment separate from Linux run reports. |
| Google Play | Ratings emphasize recent experience; version and device dimensions are available | Prefer recent, version-relevant run reports. Show sample size and report age. A favorable score for another platform or device is not compatibility evidence. |
| Microsoft Store | Ratings appear in product/search pages; recent ratings and moderated reviews | Put concise evidence on cards and details; retain report/edit controls if we later host reviews. |
| Flathub / GNOME Software | Published download statistics, metadata quality checks, shared ODRS reviews | Reuse existing statistics and investigate ODRS rather than starting an empty review community. Attribute source and period. |

Primary sources:

- Apple discovery: https://developer.apple.com/app-store/discoverability/
- Apple search: https://developer.apple.com/app-store/search/
- Steam tags: https://partner.steamgames.com/doc/store/tags
- Steam reviews: https://partner.steamgames.com/doc/store/reviews
- Steam current review API: https://partner.steamgames.com/doc/webapi/IUserReviewsService
- Google ratings: https://support.google.com/googleplay/android-developer/answer/138230?hl=en
- Microsoft ratings: https://support.microsoft.com/en-us/accounts-billing/rate-and-review-games-and-apps-in-the-microsoft-store
- Flathub statistics and quality review: https://docs.flathub.org/docs/for-app-authors/maintenance
- ODRS: https://odrs.gnome.org/ and https://odrs.gnome.org/privacy

These observations do not establish that every other store feature improves our users' task success. The borrowing decisions are our design judgment.

## Current implementation assessment

| Area | What Kompas does | Gap / next decision |
| --- | --- | --- |
| Navigation | Visible back arrow, Alt+Left, details Escape, restored underlying search/category and filters; bounded category history | Test real browse-detail-back flows and retain scroll. A complete browser-style history including nested app details and search edits is not implemented. |
| Browse | System, Flatpak and Steam filters, shared sort modes, incremental results, metadata subcategories | Steam browsing contains the fetched discovery set, not every Steam game. A scope note now points to search; do not advertise an exhaustive unified catalog. |
| Search | Name/application-ID alias/summary/description matching, a few explicit Windows alternatives and live Steam search | Application-ID aliases now improve exact-name searches. Add intent vocabulary; measure exact-name results before adding fuzzy search. Description-only matches currently add noise. |
| Homepage | Several editorial/popular/new sections and modern Steam cards; Games now starts with its filtered current Steam selection before the complete list | Too many generic sections compete. Prefer a few strong, current collections, then make all categories accessible. Never label the same list as personalized. |
| Compatibility | Package availability and architecture gates; Steam native-platform metadata; optional inclusion of Windows games | Native Linux is not a GPU/RAM/driver guarantee. Anti-cheat, Proton and hardware checks remain gaps. Unknown evidence must be shown as unknown rather than fabricated. |
| Popularity | Flathub monthly download statistics where available | Coverage and freshness vary. System packages and Steam lack equivalent download counts. Do not infer zero downloads or poor quality from missing data. |
| Ratings | No community rating integration yet | Read existing source ratings first; no invented stars and no empty submission button. |
| Install flow | Backend installs for local app sources; external storefront / launcher handoff for Steam and Heroic | Real Zorin install, launch, cancel and error-recovery testing is still required. CI startup is necessary but insufficient. |

## Three separate kinds of evidence

1. **Popularity:** a reported count for a stated provider and time period. Flathub downloads can include updates and are not an active-user count. Steam sales ranks and review counts are different measurements. A system package can be excellent without a download metric.
2. **User opinion:** ODRS stars or Steam recommendation percentage, always with provider, number of ratings and available period. No conversion of Steam percentages into invented five-star ratings, and no averaging dissimilar providers into a universal quality score.
3. **Works on Linux / this setup:** explicit platform support and relevant run reports. Separate game quality from launch failures, packaging bugs, anti-cheat and performance. A review written on Windows cannot establish Linux compatibility.

Example presentation using illustrative numbers, never production placeholders:

- `4.3/5 · 218 ODRS ratings` for application sentiment.
- `92% recommended · 12,400 Steam reviews` for game sentiment.
- `Linux native · publisher platform metadata` for platform support.
- `No recent reports for this setup` when system-specific evidence is absent.

On cards show one useful evidence line; details contain provenance, sample size, age and limitations. Missing data says unavailable, not zero stars. External failures must not block local browsing, installation or opening details.

## Ratings integration sequence

### First: reuse established communities

Prototype read-only ODRS ratings/reviews for canonical desktop application IDs. Confirm identity mappings, distributions, source versions, privacy requirements, rate limits and service terms before enabling requests. A review about the upstream application and a failure in its Flatpak build should remain distinguishable. GNOME's service supports moderation/helpfulness and locale/version context; it already has contributors.

For Steam use the current documented `IUserReviewsService/GetAppReviews/v1/` interface. The official documentation says it replaces `/appreviews`, allows anonymous reads without a key with a lower rate limit, and may cache anonymous responses for ten minutes. Handle 429 with backoff; never repeatedly query all catalog cards. Fetch lazily, cache summaries, and display provider totals and scope. Hardware-OS filters exist but the resulting sample is still not a guarantee for the local machine.

Use typed optional results with states loading, available, missing, stale and unavailable. Cache timestamps and schema version; bound network time, payload size and concurrency. Test malformed data, small samples, offline cache, expired data, mismatched app IDs and absent fields. Do not introduce a publisher API key in desktop source code.

### Then: Linux-specific experience reports

Evaluate contributing through an existing service before operating a separate backend. If a Kompas service is necessary, its useful contribution is structured Linux experience, not another undifferentiated star average:

- installed app ID, source and version;
- recommendation plus separate launch/performance status;
- distribution and desktop session; optional broad GPU/driver/Proton context;
- report date, short explanation and whether this is an update to a prior report;
- explicit preview and consent before sending any report or hardware information.

A local installed-state check can make feedback relevant, but does not prove a genuine purchaser or prevent forged submissions. Label that evidence accurately. Use one active report per pseudonymous contributor/app/source, editable and deletable; rate limits, abuse reports, moderation queue, reviewer/developer replies and burst detection. Do not upload installed-app lists, machine identifiers or usage history silently. Deletion and abuse handling are part of the feature, not later cleanup.

A local-only rating is useful as a personal note but must not be presented as a shared community score. A public review system requires an operating service, ongoing moderation and a clear privacy contract; this review does not claim to have built it.

## Ranking decisions

Hard availability constraints come first. Confirmed unavailable options are hidden or offered through an actually available alternative source. Unknown GPU/Proton status is not enough to declare an app unusable; show uncertainty and provide an explicit filter for stronger evidence.

For search, rank lexicographically: exact name/recognized application-ID alias, name prefix, other name match, useful summary/tag match, description-only match. Popularity should only break ties within a relevance tier. A popular app mentioning GIMP in a long description must not beat GIMP itself.

For discovery, compute signals within comparable providers/categories and disclose why an item appears. Include a separate new/not-yet-rated lane so established titles do not permanently crowd out new apps. Do not call lifetime totals a trend; trends need comparable recent and earlier periods. Missing data must not penalize an app as if users disliked it.

For a future best-rated sort, keep provider-specific views until there is evidence supporting cross-source normalization. For positive/negative recommendations, a Wilson lower confidence bound is a possible internal ranking signal. For star ratings, a Bayesian average with a documented category prior is a possible alternative. Display the actual provider score and sample size, not an opaque transformed score. Minimum sample sizes, age weights and priors are hypotheses to calibrate with real data rather than asserted industry constants.

## Delivery priorities

| Priority | Concrete change | Acceptance |
| --- | --- | --- |
| P0 | Back navigation and subcategories | Open an app from a filtered category/search, return without losing context; metadata subcategory excludes other genres while All retains unclassified apps. |
| P1 | Stronger exact-name/alias search and clear browse scope | GIMP, Steam, Spotify and known alternate names lead to the intended available app; a missing commercial result has a useful action rather than misleading completeness. |
| P1 | A few task collections and visible explanation | A newcomer finds a photo editor, PDF tool and controller game without choosing a package format first. Recommendations explain their basis. |
| P1 | Lazy read-only ratings with provenance | ODRS/Steam evidence shows count, provider and period; unknown/offline does not block details. |
| P2 | Favorites and later comparison | Save candidates locally without an account; return and compare price, source, compatibility and permissions. |
| P2 | Structured run reports and compatibility evidence | Reports are explicit, recent and relevant; no hidden hardware upload or unmoderated empty community. |

## Validate value with tasks

Use the same Zorin setup and configured sources for Kompas and Bazaar/Discover plus Steam. Record completion time, clicks, source jargon encountered, failed installs and wrong compatibility assumptions. Test with a newcomer as well as the maintainer; five small observed sessions are a starting point, not proof of universal superiority.

Tasks: install Spotify, find a photo editor, find a recent strategy game, find a game suitable for a controller, assess GTA's actual Linux limitations, return to a previous shortlist and recover from an unavailable source. A successful product improves these tasks; attractive screenshots alone do not establish success.
