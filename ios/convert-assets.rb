#!/usr/bin/ruby

require 'optparse'

SCRIPT_DIR = File.expand_path(File.dirname(__FILE__))
ROOT_DIR = File.dirname(SCRIPT_DIR)

# assets catalogue root
XCASSETS_DIR = File.join(SCRIPT_DIR, "MullvadVPN/Supporting Files/Assets.xcassets")

# graphical assets sources
APPICON_PATH = File.join(ROOT_DIR, "graphics/icon-square.svg")
ADDITIONAL_ASSETS_DIR = File.join(SCRIPT_DIR, "AdditionalAssets")

# app icon output
XCASSETS_APPICON_PATH = File.join(XCASSETS_DIR, "AppIcon.appiconset/AppIcon.png")
XCASSETS_APPICON_SIZE = 1024

ICON_ASSETS_DIR = File.join(
  ROOT_DIR,
  "desktop/packages/mullvad-vpn/assets/icons"
)
ICON_ASSETS = [
  'icon-account-circle.svg',
  'icon-add-circle.svg',
  'icon-alert-circle.svg',
  'icon-checkmark-circle.svg',
  'icon-checkmark.svg',
  'icon-chevron-down-circle.svg',
  'icon-chevron-down.svg',
  'icon-chevron-left-circle.svg',
  'icon-chevron-left.svg',
  'icon-chevron-right-circle.svg',
  'icon-chevron-right.svg',
  'icon-chevron-up-circle.svg',
  'icon-chevron-up.svg',
  'icon-copy.svg',
  'icon-cross-circle.svg',
  'icon-cross.svg',
  'icon-edit-circle.svg',
  'icon-external.svg',
  'icon-filter-circle.svg',
  'icon-grabber.svg',
  'icon-hide.svg',
  'icon-info-circle.svg',
  'icon-more-horizontal-circle.svg',
  'icon-more-horizontal.svg',
  'icon-more-vertical-circle.svg',
  'icon-more-vertical.svg',
  'icon-reconnect.svg',
  'icon-remove-circle.svg',
  'icon-search-circle.svg',
  'icon-search.svg',
  'icon-settings-filled.svg',
  'icon-show.svg',
]

IMAGE_ASSETS_DIR = File.join(
  ROOT_DIR,
  "desktop/packages/mullvad-vpn/assets/images"
)
IMAGE_ASSETS = [
  "daita-off-illustration.svg",
  "daita-on-illustration.svg",
  "location-marker-secure.svg",
  "location-marker-unsecure.svg",
  "multihop-illustration.svg",
  "negative.svg",
  "positive.svg",
  "spinner.svg",
]

# graphical assets to resize.
RESIZE_ASSETS = {
  "icon-info-circle.svg" => ["icon-info-circle.svg", 18, 18],
  "icon-checkmark.svg" => ["icon-checkmark-sml.svg", 16, 16]
}

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

# SVG conversion tool environment variables.
SVG_CONVERT_ENVIRONMENT_VARIABLES = {
  # Fix PDF "CreationDate" for reproducible output
  "SOURCE_DATE_EPOCH" => "1596022781"
}

# Fix DPI at 72 to produce the same size assets as defined in SVG files (in pixels)
SVG_CONVERT_DEFAULT_OPTIONS = ["--dpi-x=72", "--dpi-y=72"]

# Functions
def generate_graphical_assets(assets, asset_dir)
  for asset_name in assets do
    svg_file = File.join(asset_dir, asset_name)
    image_name = pascal_case(File.basename(svg_file, ".svg"))
    output_dir = File.join(XCASSETS_DIR, "#{image_name}.imageset")

    if !Dir.exists?(output_dir)
      puts "Create directory #{output_dir}"
      Dir.mkdir(output_dir)
    end

    output_file = File.join(output_dir, "#{image_name}.pdf")

    puts "Convert #{svg_file} -> #{output_file}"
    rsvg_convert("--format=pdf", svg_file, "--output", output_file)
  end
end

def generate_resized_assets()
  RESIZE_ASSETS.each do |asset_name, resize_options|
    (new_asset_name, width, height) = resize_options

    svg_file = File.join(ICON_ASSETS_DIR, asset_name)
    image_name = pascal_case(File.basename(new_asset_name, ".svg"))
    output_dir = File.join(XCASSETS_DIR, "#{image_name}.imageset")

    if !Dir.exists?(output_dir)
      puts "Create directory #{output_dir}"
      Dir.mkdir(output_dir)
    end

    output_file = File.join(output_dir, "#{image_name}.pdf")

    puts "Convert and resize #{svg_file} -> #{output_file} (#{width} x #{height})"
    rsvg_convert("--width=#{width}", "--height=#{height}", "--format=pdf", svg_file, "--output", output_file)
  end
end

def generate_app_icon()
  image_name = File.basename(XCASSETS_APPICON_PATH, ".png")
  puts "Generate #{image_name} -> #{XCASSETS_APPICON_PATH}"
  rsvg_convert("--width=#{XCASSETS_APPICON_SIZE}", "--height=#{XCASSETS_APPICON_SIZE}", "--format=png", APPICON_PATH, "--output", XCASSETS_APPICON_PATH)
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
    rsvg_convert("--format=pdf", svg_file, "--output", output_file)
  end
end

def rsvg_convert(*args)
  command = ["rsvg-convert", *SVG_CONVERT_DEFAULT_OPTIONS, *args]
  system(SVG_CONVERT_ENVIRONMENT_VARIABLES, *command)
end

def pascal_case(str)
  return str.split('-').collect(&:capitalize).join
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
    generate_app_icon
  end

  opts.on("--import-desktop-assets", "Import assets from the desktop app") do |v|
    generate_graphical_assets(ICON_ASSETS, ICON_ASSETS_DIR)
    generate_graphical_assets(IMAGE_ASSETS, IMAGE_ASSETS_DIR)
    generate_resized_assets
  end

  opts.on("--additional-assets", "Generate additional assets") do |v|
    generate_additional_assets
  end

  opts.on_tail("-h", "--help", "Show this message") do
    puts opts
    exit
  end
end.parse!
