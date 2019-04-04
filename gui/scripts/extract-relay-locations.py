import os
from os import path
import urllib2
import json
from polib import POFile, POEntry

# import order is important, see https://github.com/Toblerity/Shapely/issues/553
from shapely.geometry import shape, mapping
import fiona

SCRIPT_DIR = path.dirname(path.realpath(__file__))
LOCALE_DIR = path.normpath(path.join(SCRIPT_DIR, "../locales"))
OUT_DIR = path.join(SCRIPT_DIR, "out")
LOCALE_OUT_DIR = path.join(OUT_DIR, "locales")

def pretty_json(dict):
  return json.dumps(dict, indent=2, sort_keys=True)

def get_shape_path(dataset_name):
  return path.join(SCRIPT_DIR, dataset_name, dataset_name + ".shp")

def lower_dict_keys(input_dict):
  return dict((k.lower(), v) for k, v in input_dict.iteritems())

def convert_locale_ident(locale_ident):
  return locale_ident.replace("-", "_")

def get_locale_language(locale_ident):
  return locale_ident.split("-")[0]

def find_translation(fiona_source, locale, english_city_name):
  name_key = "_".join(("name", get_locale_language(locale)))
  name_alt_key = "_".join(("name", convert_locale_ident(locale)))

  for feat in fiona_source:
    props = lower_dict_keys(feat["properties"])

    # namepar works for Wien
    # use nameascii to match Sao Paolo
    if props.get("name") == english_city_name or props.get("namepar") == english_city_name or props.get("nameascii") == english_city_name:
      match_keys = (get_locale_language(locale), convert_locale_ident(locale))

      for key in map(lambda k: "name_" + k, match_keys):
        value = props.get(key)

        if value is not None:
          return value

      print "Cannot find the translation for {}. Probe keys: {}".format(locale, match_keys)

  return ""


def request_relays():
  data = '{"jsonrpc": "2.0", "id": "0", "method": "relay_list_v2"}'
  headers = { "Content-Type": 'application/json' }
  request = urllib2.Request("https://api.mullvad.net/rpc/", data, headers)
  return json.load(urllib2.urlopen(request))

def extract_pot(countries):
  pot = POFile(encoding='UTF-8')
  output_path = path.join(LOCALE_OUT_DIR, "relay-locations.pot")

  for country in countries:
    cities = country.get("cities")
    if cities is not None:
      for city in cities:
        city_name = city.get("name")
        if city_name is not None:
          entry = POEntry(
            msgid=city_name,
            msgstr=u"",
            comment=u"{} {}".format(country.get("code"), city.get("code"))
          )
          pot.append(entry)
          print u"  {} ({})".format(city["name"], city["code"]).encode('utf-8')

  pot.save(output_path)


def extract_po(fiona_source, countries, locale):
  po = POFile(encoding='UTF-8')
  locale_out_dir = path.join(LOCALE_OUT_DIR, locale)
  output_path = path.join(locale_out_dir, "relay-locations.po")

  if not path.exists(locale_out_dir):
    os.makedirs(locale_out_dir)

  for country in countries:
    country_code = country.get("code")
    cities = country.get("cities")

    if cities is not None:
      for city in cities:
        city_name = city.get("name")

        # Make sure to append the US state back to the translated name of the city
        if country_code == "us":
          split = city_name.rsplit(",", 2)
          translated_name = find_translation(fiona_source, locale, split[0].strip())
          if translated_name != "" and len(split) > 1:
            translated_name = u"{}, {}".format(translated_name, split[1].strip())
        else:
          translated_name = find_translation(fiona_source, locale, city_name)

        if city_name is not None:
          entry = POEntry(
            msgid=city_name,
            msgstr=translated_name,
            comment=u"{} {}".format(country.get("code"), city.get("code"))
          )
          po.append(entry)
          print u"  {} ({}) -> \"{}\"".format(city["name"], city["code"], translated_name).encode('utf-8')

  po.save(output_path)


def extract_translations(countries):
  geo_source = get_shape_path("ne_50m_populated_places")

  print "Generating relay-locations.pot"
  extract_pot(countries)

  with fiona.collection(geo_source, "r") as source:
    for locale in os.listdir(LOCALE_DIR):
      locale_dir = path.join(LOCALE_DIR, locale)
      if path.isdir(locale_dir):
        print "Generating {}/relay-locations.po".format(locale)
        extract_po(source, countries, locale)


try:
  response = request_relays()
except Exception as e:
  print "Failed to fetch the relays list: {}".format(e)

result = response.get("result")
if result is not None:
  countries = result.get("countries")
  if countries is not None:
    extract_translations(countries)
  else:
    print "Missing the countries field.".format(pretty_json(response))
else:
  print "Missing the result field\n\nResponse: {}".format(pretty_json(response))
