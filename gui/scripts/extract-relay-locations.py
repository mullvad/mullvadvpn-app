from os import path
import urllib2
import json
from polib import POFile, POEntry

SCRIPT_DIR = path.dirname(path.realpath(__file__))
LOCALE_DIR = path.normpath(path.join(SCRIPT_DIR, "../locales"))

def pretty_json(dict):
  return json.dumps(dict, indent=2)

def request_relays():
  data = '{"jsonrpc": "2.0", "id": "0", "method": "relay_list_v2"}'
  headers = { "Content-Type": 'application/json' }
  request = urllib2.Request("https://api.mullvad.net/rpc/", data, headers)
  return json.load(urllib2.urlopen(request))

try:
  response = request_relays()
except Exception as e:
  print "Failed to fetch the relays list: {}".format(e)

result = response.get("result")
if result is not None:
  countries = result.get("countries")
  if countries is not None:
    for country in countries:
      cities = country.get("cities")
      if cities is not None:
        for city in cities:
          print(u"{} ({})".format(city["name"], city["code"]))
      else:
        print("Country {} has no cities?", country.get("name"))
  else:
    print("Missing the countries field.".format(pretty_json(response)))
else:
  print("Missing the result field\n\nResponse: {}".format(pretty_json(response)))
