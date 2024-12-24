# Install Guide
### Plugin
Add `https://aetherment.sevii.dev/plugin` to your Custom Plugin Repositories in `/xlsettings` > Experimental
#### Linux specific steps
One required library `bcryptprimitives.dll` is not included in XLCore's Wineprefix.
1. Make sure wine is installed on your system (tested and working with `wine-9.14` and `wine-9.16`, not working with `wine-GE-8.26`)
2. Open a terminal
3. Enter the following command `wine ""; cp "$HOME/.wine/drive_c/windows/system32/bcryptprimitives.dll" "$HOME/.xlcore/wineprefix/drive_c/windows/system32/bcryptprimitives.dll"`
### Desktop Client
Download the latest version for your system from the releases tab.
NOTE: this currently does not have much functionality besides mod creation through the gui tools tab and CLI.

# Support
If you wish to support me and my work, you can do so [here](https://buymeacoffee.com/sevii77)
