require "fileutils"


dirs = Dir.glob("*").filter {|d| File.directory?(d) && d != "login" }

dirs.each do |d|

  dir = "#{d}/api/src/main/kotlin/net/mullvad/mullvadvpn/feature/#{d}/api"
  file = "#{d.capitalize}NavKey.kt" 
  FileUtils.mkdir_p(dir)

  code = "package net.mullvad.mullvadvpn.feature.#{d}.api

import androidx.navigation3.runtime.NavKey
import kotlinx.serialization.Serializable

@Serializable
object #{d.capitalize}NavKey : NavKey
"

  gradle = "plugins {
    alias(libs.plugins.mullvad.android.library.feature.api)
}

android {
    namespace = \"net.mullvad.mullvadvpn.feature.#{d}.api\"
}"

  File.write(File.join(d, "api/build.gradle.kts"), gradle)
  #File.write(File.join(dir, file), code)


end
