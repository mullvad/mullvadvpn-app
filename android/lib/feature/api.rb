require "fileutils"


dirs = Dir.glob("*").filter {|d| File.directory?(d) && d != "login" }

dirs.each do |d|

  dir = "#{d}/impl/src/main/kotlin/net/mullvad/mullvadvpn/feature/#{d}/impl/navigation"

  next if File.directory?(dir)
  FileUtils.mkdir_p(dir)

  file = "#{d.capitalize}EntryProvider.kt"

  code = "package net.mullvad.mullvadvpn.feature.#{d}.impl.navigation

import androidx.navigation3.runtime.EntryProviderScope
import androidx.navigation3.runtime.NavKey
import net.mullvad.mullvadvpn.core.nav3.Navigator

fun EntryProviderScope<NavKey2>.create#{d.capitalize}Entry(navigator: Navigator) {
    entry<#{d.capitalize}NavKey> { #{d.capitalize}(navigator = navigator) }
}
"

  File.write(File.join(dir, file), code)

end
