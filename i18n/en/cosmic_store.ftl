app-name = Kompas
back = Back
cancel = Cancel
sort-relevance = Relevance
sort-popular = Most Popular
sort-recent = Recently Updated
sort-wayland = Best Wayland Support
filter-all = All Apps
filter-excellent = Excellent Wayland Support
filter-good = Good Wayland Support
filter-caution = Caution - May Have Issues
filter-limited = Limited Wayland Support
filter-unknown = Unknown Compatibility
editors-choice-tooltip = Editor's Choice
verified-tooltip = Verified
monthly-downloads-tooltip = Monthly downloads
check-for-updates = Check for updates
checking-for-updates = Checking for updates...
close = Close
install = Install
no-installed-applications = No installed applications.
no-updates = All installed applications are up to date.
no-results = No results for "{$search}".
notification-in-progress = Installations and updates are in progress.
open = Open
see-all = See all
uninstall = Uninstall
update = Update
update-all = Update all
place-on-desktop = Place on desktop
place-applet = Place applet
place-applet-desc = Choose where to add the applet before refining its position.
panel = Panel
dock = Dock
place-and-refine = Place and refine

# Codec dialog
codec-title = Install additional packages?
codec-header = "{$application}" requires additional packages providing "{$description}".
codec-footer =
    The use of these additional packages may be restricted in some countries.
    You must verify that one of the following is true:
     • These restrictions do not apply in your country of legal residence
     • You have permission to use this software (for example, a patent license)
     • You are using this software for research purposes only
codec-error = There were errors during package installation.
codec-installed = The packages have been installed.

# Progress footer
details = Details
dismiss = Dismiss message
operations-running = {$running} operations running ({$percent}%)...
operations-running-finished = {$running} operations running ({$percent}%), {$finished} finished...

# Repository add error dialog
repository-add-error-title = "Failed to add repository"

# Repository remove dialog
repository-remove-title = Remove "{$name}" repository?
repository-remove-body = Removing this repository will { $dependency ->
    [none] delete
    *[other] remove "{$dependency}" and delete
} the following applications and items. They will need to be reinstalled if the repository is added again.
add = Add
adding = Adding...
remove = Remove
removing = Removing...
loading = Loading...

# Uninstall Dialog
uninstall-app = Uninstall {$name}?
uninstall-app-warning = Uninstalling {$name} will delete its data.
uninstall-app-flatpak-warning = Uninstalling {$name} will keep its documents and data.
delete-app-data = Permanently delete app data

# Nav Pages
explore = Explore
create = Create
work = Work
develop = Develop
learn = Learn
game = Game
relax = Relax
socialize = Socialize
utilities = Utilities
applets = Applets
installed-apps = Installed apps
updates = Updates

## Applets page
enable-flathub-cosmic = Please enable Flathub and COSMIC Flatpak to see available applets.
manage-repositories = Manage repositories
editors-choice = Editor's Choice

# Explore Pages

verified = Verified
popular-apps = Popular apps
made-for-cosmic = Made for COSMIC
new-apps = New releases on Steam
recently-updated = Recently updated
development-tools = Development tools
scientific-tools = Scientific tools
productivity-apps = Productivity apps
graphics-and-photography-tools = Graphics & photography tools
social-networking-apps = Social networking apps
games = Games
music-and-video-apps = Music & video apps
apps-for-learning = Apps for learning

# Details Page
addons = Addons
source-installed = {$source} (installed)
developer = Developer
app-developers = {$app} Developers
monthly-downloads = Flathub monthly downloads
version = Version {$version}
licenses = Licenses
proprietary = Proprietary
view-more = View more

## App URLs
bug-tracker = Bug tracker
contact = Contact
donation = Donation
faq = FAQ
help = Help
homepage = Homepage
translate = Translate

# Context Pages

## Operations
cancelled = Cancelled
operations = Operations
no-operations = No operations in history.
pending = Pending
failed = Failed
complete = Complete

## Settings
settings = Settings

## Release notes
latest-version = Latest version
no-description = No description available.

## Repositories
recommended-flatpak-sources = Recommended Flatpak sources
custom-flatpak-sources = Custom Flatpak sources
import-flatpakrepo = Import .flatpakrepo file to add a custom source
no-custom-flatpak-sources = No custom Flatpak sources
import = Import
no-flatpak = No flatpak support
software-repositories = Software repositories

### Appearance
appearance = Appearance
theme = Theme
match-desktop = Match desktop
dark = Dark
light = Light

# Wayland compatibility
compatibility-warning = Compatibility Warning
x11-only-tooltip = X11 Only - May not work on Wayland
x11-only-description = This application only supports X11 and may not work properly on COSMIC desktop (Wayland). You may experience issues with window management, file pickers, or the app may not start at all.
wayland-issues-warning = Potential Wayland Issues
wayland-issues-description = This application uses {$framework} which may have compatibility issues on Wayland.
framework-qtwebengine = Qt WebEngine
framework-electron = Electron
wayland-native = Wayland Native
wayland-native-tooltip = Metadata indicates native Wayland support
wayland-estimate-tooltip = Estimated from metadata; not verified on your system.
wayland-moderate-tooltip = Moderate estimated Wayland risk
system-packages = System packages (Zorin / Ubuntu)
store-welcome = Discover something worth installing
store-intro = Apps from your system and Flathub, plus current games from Steam.
steam-native = Native Linux version listed
steam-unverified = Windows version · check Proton compatibility
steam-free = Free
steam-check-price = See current price in Steam
steam-handoff-description = Steam handles purchase, ownership and installation. Prices shown are for the Netherlands and may change. Windows support does not guarantee that a game works with Proton; check the specific game and game mode.
steam-controller = Controller support reported by Steam
steam-install = Install via Steam
steam-store = View / buy in Steam
steam-compatibility = Check ProtonDB
alternative-results = Available alternatives from your configured software sources:

all-apps = All apps
source-all = All sources
sort-name = Name (A–Z)
filter-source = Show apps from
filter-sort = Sort by
reset-filters = Reset filters
native-linux-only = Native Linux only
native-linux-help = Hide Windows games requiring Proton. Native support does not guarantee that your hardware meets the requirements.
catalog-loading = Gathering apps available on this computer…
catalog-empty = Nothing here with these filters
catalog-empty-help = Try another source or allow games requiring Proton. Check your software sources if you are missing apps.
result-count = { $count } results
results-shown = Showing { $shown } of { $total }
show-more = Show more
search-title = Results for “{ $search }”
discover-steam = Find Steam
discover-heroic = Epic & GOG with Heroic
game-launcher-help = Install your launcher here, then browse and play through its own store.
steam-required = Install Steam first, then return to this game.
steam-get-client = Find and install Steam
steam-ready = Steam is available. It handles ownership and installation.

search-store = Search apps and games

search-steam-loading = Searching Steam… Your local apps are already shown.
