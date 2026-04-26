# Maintainer: Tritao <274685803+tritaum@users.noreply.github.com>
pkgname=patched-mullvad-vpn-daemon-bin
pkgver=2026.2.beta1
pkgrel=1
_appver=${pkgver/.beta/-beta}
pkgdesc="Mullvad VPN daemon and CLI patched with IP range and Netbird/Tailscale Split-Tunneling support"
arch=('x86_64')
url="https://github.com/tritaum/ip-split-tunneling-mullvad"
license=('GPL-3.0-or-later')
depends=(
  'dbus'
  'iputils'
  'libnftnl'
)
provides=('mullvad-vpn-daemon')
conflicts=('mullvad-vpn-daemon' 'mullvad-vpn-daemon-bin' 'patched-mullvad-vpn-daemon')
install='patched-mullvad-vpn-daemon.install'
source_x86_64=("$url/releases/download/$_appver/mullvad-vpn-daemon_${_appver}_amd64.deb")
sha256sums_x86_64=('SKIP')

package() {
  bsdtar -xvf data.tar.xz -C "$pkgdir/"

  ln -sf "/opt/Mullvad VPN/resources/mullvad-problem-report" "$pkgdir/usr/bin/"

  install -dm755 "$pkgdir/usr/share/zsh/site-functions"
  if [[ -d "$pkgdir/usr/local/share/zsh/site-functions" ]]; then
    mv "$pkgdir/usr/local/share/zsh/site-functions/_mullvad" \
      "$pkgdir/usr/share/zsh/site-functions/"
    rm -rf "$pkgdir/usr/local"
  fi
}
