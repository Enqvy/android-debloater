# android debloater

removes bloatware from android phones without root

## what it does

- connects to android via usb or wifi
- lists all the crap apps on your phone
- removes them (or disables if removal fails)
- has a built-in list of common bloatware
- can backup what you removed
- can restore apps if you mess up

basically uninstalls system apps without needing root access. uses adb under the hood.

## requirements
 
- adb (android debug bridge)
  - the tool will offer to install it for you if its missing
  - works with apt, pacman, dnf, nix, homebrew, winget, chocolatey
- android phone with usb debugging enabled
  - go to settings -> about phone -> tap build number 7 times
  - then settings -> developer options -> enable usb debugging

## install

download the binary for your system from releases or build it yourself

### building from source

```bash
cargo build --release