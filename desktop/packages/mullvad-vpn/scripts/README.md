This is a folder with the supporting scripts written in Python 3, node, bash.


## Maps and location translations

### Dependency installation notes

1. Run the following command in terminal to install python dependencies:
   `pip3 install -r requirements.txt`

2. Make sure you have gettext utilities installed.
   https://www.gnu.org/software/gettext/


### Update translations

See [locales/README.md](../locales/README.md) for information about how to handle translations.

The `<repo>/scripts/localization` script is, among other things, calling into `fetch-relay-locations.py`
and `integrate-relay-locations.py` in this directory.

* `fetch-relay-locations.py` fetches the relay list and extracts all country and city names.
* `intregrate-relay-locations.py` integrates the fetched relay locations into
`../locales/relay-locations.pot`.

### Locking Python dependencies

1. Freeze dependencies:

```
pip3 freeze -r requirements.txt
```

and save the output into `requirements.txt`.


2. Hash them with `hashin` tool:

```
hashin --python 3.7 --verbose --update-all
```
