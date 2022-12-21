#!/usr/bin/ruby

require 'optparse'

SCRIPT_DIR = File.expand_path(File.dirname(__FILE__))
ROOT_DIR = File.dirname(SCRIPT_DIR)

# assets catalogue root
XCASSETS_DIR = File.join(SCRIPT_DIR, "MullvadVPN/Assets.xcassets")

# graphical assets sources
APPICON_PATH = File.join(ROOT_DIR, "graphics/icon-square.svg")
GRAPHICAL_ASSETS_DIR = File.join(ROOT_DIR, "gui/assets/images")
ADDITIONAL_ASSETS_DIR = File.join(SCRIPT_DIR, "AdditionalAssets")

# app icon output
XCASSETS_APPICON_PATH = File.join(XCASSETS_DIR, "AppIcon.appiconset/AppIcon.png")
XCASSETS_APPICON_SIZE = 1024

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
  "icon-close.svg",
  "icon-close-sml.svg",
  "icon-copy.svg",
  "icon-obscure.svg",
  "icon-unobscure.svg",
]

# graphical assets to resize.
RESIZE_ASSETS = {
   "icon-tick.svg" => ["icon-tick-sml.svg", 16, 16],
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

def generate_resized_assets()
  RESIZE_ASSETS.each do |asset_name, resize_options|
    (new_asset_name, width, height) = resize_options

    svg_file = File.join(GRAPHICAL_ASSETS_DIR, asset_name)
    image_name = pascal_case(File.basename(new_asset_name, ".svg"))
    output_dir = File.join(XCASSETS_DIR, "#{image_name}.imageset")

    if !Dir.exists?(output_dir)
      puts "Create directory #{output_dir}"
      Dir.mkdir(output_dir)
    end

    output_file = File.join(output_dir, "#{image_name}.pdf")

    puts "Convert and resize #{svg_file} -> #{output_file} (#{width} x #{height})"
    system(SVG_CONVERT_ENVIRONMENT_VARIABLES, "rsvg-convert", "--width=#{width}", "--height=#{height}", "--format=pdf", svg_file, "--output", output_file)
  end
end

def genereate_app_icon()
  image_name = File.basename(XCASSETS_APPICON_PATH, ".png")
  puts "Generate #{image_name} -> #{XCASSETS_APPICON_PATH}"
  system("rsvg-convert", "--width=#{XCASSETS_APPICON_SIZE}", "--height=#{XCASSETS_APPICON_SIZE}", "--format=png", APPICON_PATH, "--output", XCASSETS_APPICON_PATH)
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
