#!/usr/bin/ruby

require 'optparse'

SCRIPT_DIR = File.expand_path(File.dirname(__FILE__))
ROOT_DIR = File.dirname(SCRIPT_DIR)

# assets catalogue root
XCASSETS_DIR = File.join(SCRIPT_DIR, "MullvadVPN/Assets.xcassets")
XCASSETS_APPICON_DIR = File.join(XCASSETS_DIR, "AppIcon.appiconset")

# graphical assets sources
APPICON_PATH = File.join(ROOT_DIR, "graphics/icon-square.svg")
GRAPHICAL_ASSETS_DIR = File.join(ROOT_DIR, "gui/assets/images")
ADDITIONAL_ASSETS_DIR = File.join(SCRIPT_DIR, "AdditionalAssets")

# graphical assets to import
GRAPHICAL_ASSETS = [
  "icon-arrow.svg",
  "icon-back.svg",
  "icon-chevron-down.svg",
  "icon-chevron-up.svg",
  "icon-chevron.svg",
  "icon-extLink.svg",
  "icon-fail.svg",
  "icon-reload.svg",
  "icon-settings.svg",
  "icon-spinner.svg",
  "icon-success.svg",
  "icon-tick.svg",
  "location-marker-secure.svg",
  "location-marker-unsecure.svg",
  "logo-icon.svg",
  "logo-text.svg",
  "icon-close-sml.svg"
]

# App icon sizes
APP_ICON_SIZES = [
  # iphone-notification 20pt at 2x, 3x
  ["AppIconPhoneNotification", 20, 2, 3],

  # iphone-settings at 29pt, 2x, 3x
  ["AppIconPhoneSettings", 29, 2, 3],

  # iphone-spotlight at 40pt, 2x, 3x
  ["AppIconPhoneSpotlight", 40, 2, 3],

  # iphone-app at 60pt, 2x, 3x
  ["AppIconPhone", 60, 2, 3],

  # ipad-notifications at 20pt, 1x, 2x
  ["AppIconPadNotifications", 20, 1, 2],

  # ipad-settings at 29pt, 1x, 2x
  ["AppIconPadSettings", 29, 1, 2],

  # ipad-spotlight at 40pt, 1x, 2x
  ["AppIconPadSpotlight", 40, 1, 2],

  # ipad-app at 76pt, 1x, 2x
  ["AppIconPad", 76, 1, 2],

  # ipad-pro-app at 83.5pt, 2x
  ["AppIconPadPro", 83.5, 2],

  # appstore-ios (marketing) at 1024pt, 1x
  ["AppStoreIosMarketing", 1024, 1],
]

# Additional assets generated from SVG -> vector PDF
ADDITIONAL_ASSETS = [
  "DefaultButton.svg",
  "SuccessButton.svg",
  "DangerButton.svg",
  "TranslucentDangerButton.svg",
  "TranslucentNeutralButton.svg",
  "TranslucentDangerSplitLeftButton.svg",
  "TranslucentDangerSplitRightButton.svg",
  "IconBackTransitionMask.svg"
]

# SVG convertion tool environment variables. 
SVG_CONVERT_ENVIRONMENT_VARIABLES = {
  # Fix PDF "CreationDate" for reproducible output 
  "SOURCE_DATE_EPOCH" => "1596022781"
}

# Functions

def generate_graphical_assets()
  for asset_name in GRAPHICAL_ASSETS do
    svg_file = File.join(GRAPHICAL_ASSETS_DIR, asset_name)
    image_name = pascal_case(File.basename(svg_file, ".svg"))
    output_dir = File.join(XCASSETS_DIR, "#{image_name}.imageset")

    if !Dir.exists?(output_dir)
      puts "Create directory #{output_dir}"
      Dir.mkdir(output_dir)
    end

    output_file = File.join(output_dir, "#{image_name}.pdf")

    puts "Convert #{svg_file} -> #{output_file}"
    system(SVG_CONVERT_ENVIRONMENT_VARIABLES, "rsvg-convert", "--format=pdf", svg_file, "--output", output_file)
  end
end

def genereate_app_icon()
  for (icon_name, nominal_size, *retina_scales) in APP_ICON_SIZES do
    for retina_scale in retina_scales do
      scale_suffix = retina_scale_suffix(retina_scale)
      output_file = File.join(XCASSETS_APPICON_DIR, "#{icon_name}#{scale_suffix}.png")
      actual_size = (nominal_size * retina_scale).to_i

      puts "Generate #{icon_name}: #{nominal_size} (#{retina_scale}x) -> #{output_file}"
      system("rsvg-convert", "--width=#{actual_size}", "--height=#{actual_size}", "--format=png", APPICON_PATH, "--output", output_file)
    end
  end
end

def generate_additional_assets()
  for asset_name in ADDITIONAL_ASSETS do
    svg_file = File.join(ADDITIONAL_ASSETS_DIR, asset_name)
    image_name = File.basename(svg_file, ".svg")
    output_dir = File.join(XCASSETS_DIR, "#{image_name}.imageset")
    output_file = File.join(output_dir, "#{image_name}.pdf")

    if !Dir.exists?(output_dir)
      puts "Create directory #{output_dir}"
      Dir.mkdir(output_dir)
    end

    puts "Generate #{image_name} -> #{output_file}"
    system(SVG_CONVERT_ENVIRONMENT_VARIABLES, "rsvg-convert", "--format=pdf", svg_file, "--output", output_file)
  end
end

def pascal_case(str)
  return str.split('-').collect(&:capitalize).join
end

def retina_scale_suffix(retina_scale)
  return retina_scale == 1 ? "" : "@#{retina_scale}x"
end

def command?(name)
  `which #{name}`
  $?.success?
end

# Check requirements

if !command?("rsvg-convert")
  puts "rsvg-convert is not installed."
  exit
end

# Parse program arguments

ARGV << '-h' if ARGV.empty?

OptionParser.new do |opts|
  opts.banner = "Usage: convert-assets.rb [options]"

  opts.on("--app-icon", "Generate application icon assets") do |v|
    genereate_app_icon
  end

  opts.on("--import-desktop-assets", "Import assets from the desktop app") do |v|
    generate_graphical_assets
  end

  opts.on("--additional-assets", "Generate additional assets") do |v|
    generate_additional_assets
  end

  opts.on_tail("-h", "--help", "Show this message") do
    puts opts
    exit
  end
end.parse!
