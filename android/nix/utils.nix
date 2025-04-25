{ lib, ... }:

let
  extractBuildToolsVersion = fileContent:
    let
      regex = "const val buildToolsVersion = \"(\\d+\\.\\d+\\.\\d+)\"";
      match = builtins.match regex fileContent;
    in
      if match != null then match[1]
      else throw "Error: buildToolsVersion could not be extracted from Versions.kt";

  extractCompileSdkVersion = fileContent:
    let
      regex = "const val compileSdkVersion = (\\d+)";
      match = builtins.match regex fileContent;
    in
      if match != null then match[1]
      else throw "Error: compileSdkVersion could not be extracted from Versions.kt";

  extractNdkVersion = fileContent:
    let
      regex = "const val ndkVersion = \"(\\d+\\.\\d+\\.\\d+)\""; # Adjust regex if needed
      match = builtins.match regex fileContent;
    in
      if match != null then match[1]
      else throw "Error: ndkVersion could not be extracted from Versions.kt";

in
  {
    extractBuildToolsVersion = extractBuildToolsVersion;
    extractCompileSdkVersion = extractCompileSdkVersion;
    extractNdkVersion = extractNdkVersion;
  }
