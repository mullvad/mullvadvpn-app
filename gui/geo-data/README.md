This is a folder with python 2 and node scripts to produce geographical
data for the Mullvad VPN app.


## Dependency installation notes

1. Run the following command in terminal to install python dependencies:
`pip install -r requirements.txt`

2. Run `npm install -g topojson-server` to install `geo2topo` tool which is
used by python scripts to convert GeoJSON to TopoJSON


## Geo data installation notes

Go to http://www.naturalearthdata.com/downloads/50m-cultural-vectors/ and
download ZIP files with the following shapes:

- Admin 0 – Countries
- Admin 1 – States, provinces - boundary lines
- Populated Places - simple dataset is enough

or use cURL to download all ZIPs:

```
curl -L -O http://www.naturalearthdata.com/http//www.naturalearthdata.com/download/50m/cultural/ne_50m_admin_0_countries.zip
curl -L -O http://www.naturalearthdata.com/http//www.naturalearthdata.com/download/50m/cultural/ne_50m_admin_1_states_provinces_lines.zip
curl -L -O http://www.naturalearthdata.com/http//www.naturalearthdata.com/download/50m/cultural/ne_50m_populated_places_simple.zip
```

Extract the downloaded ZIP files into geo-data.
Make sure the following folders exist after extraction:

- ne_50m_admin_0_countries
- ne_50m_admin_1_states_provinces_lines
- ne_50m_populated_places_simple

or use the following script:

```
mkdir ne_50m_admin_1_states_provinces_lines
mkdir ne_50m_populated_places_simple
mkdir ne_50m_admin_0_countries

unzip ne_50m_admin_0_countries.zip -d ne_50m_admin_0_countries
unzip ne_50m_admin_1_states_provinces_lines.zip -d ne_50m_admin_1_states_provinces_lines
unzip ne_50m_populated_places_simple.zip -d ne_50m_populated_places_simple
```

## Geo data extraction notes

Run the following script to produce a TopoJSON data used by the app:

```
python extract-geo-data.py
```

and finally generate the R-Tree cache:

```
npx babel-node prepare-rtree.js
```

At this point all of the data should be saved in `gui/geo-data/out` folder.

## App integration notes

Once you've extracted all the geo data, run the integration script that will
copy all files ignoring intermediate ones into the `gui/src/assets/geo` folder:

```
python integrate-into-app.py
```
